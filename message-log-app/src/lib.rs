use std::time::{SystemTime, UNIX_EPOCH};
use hyperprocess_macro::hyperprocess;
use hyperware_process_lib::{
    LazyLoadBlob, get_blob, last_blob,
    http::server::{
        HttpBindingConfig, HttpServer, HttpServerRequest, StatusCode, 
        WsMessageType, WsBindingConfig, send_response, send_ws_push
    },
    logging::{error, info, init_logging, Level},
    Address, Binding, SaveOptions
};
use serde::{Serialize, Deserialize};
use anyhow::anyhow;
mod types;
use types::{MessageChannel, MessageType, LogEntry, StatusResponse, HistoryResponse, SuccessResponse, ErrorResponse};

wit_bindgen::generate!({
    path: "target/wit",
    world: "message-log-app-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});
// Configuration for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Maximum number of messages to keep in history
    pub max_history: usize,
    /// Whether to log message content
    pub log_content: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_history: 100,
            log_content: true,
        }
    }
}

/// Represents the application state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppState {
    /// Tracks message history for all channels
    pub message_history: Vec<LogEntry>,
    /// Message counts by channel
    pub message_counts: Vec<(MessageChannel, usize)>,
    /// Configuration settings
    pub config: AppConfig,
    /// Connected WebSocket clients (channel_id -> path)
    pub connected_clients: Vec<(u32, String)>,
}

// Helper function to get current timestamp
fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// Helper function to log a message and update counts
fn log_message(
    state: &mut AppState,
    source: String,
    channel: MessageChannel,
    message_type: MessageType,
    content: Option<String>,
) {
    // Add to message history
    state.message_history.push(LogEntry {
        source,
        channel: format!("{:?}", channel),
        type_name: format!("{:?}", message_type),
        content: if state.config.log_content { content } else { None },
        timestamp: get_timestamp(),
    });
    
    // Update message count for this channel
    state.increment_channel_count(channel);
    
    // Trim history if needed
    if state.message_history.len() > state.config.max_history {
        state.message_history.remove(0);
    }
}

impl AppState {
    /// Increment count for a channel
    pub fn increment_channel_count(&mut self, channel: MessageChannel) {
        if let Some(count) = self.message_counts.iter_mut().find(|(ch, _)| *ch == channel) {
            count.1 += 1;
        } else {
            self.message_counts.push((channel, 1));
        }
    }

    /// Add a client connection
    pub fn add_client(&mut self, channel_id: u32, path: String) {
        self.connected_clients.push((channel_id, path));
    }

    /// Remove a client connection
    pub fn remove_client(&mut self, channel_id: u32) {
        self.connected_clients.retain(|(id, _)| *id != channel_id);
    }

    /// Get client path
    pub fn get_client_path(&self, channel_id: u32) -> Option<&str> {
        self.connected_clients
            .iter()
            .find(|(id, _)| *id == channel_id)
            .map(|(_, path)| path.as_str())
    }

    /// Clear message counts
    pub fn clear_counts(&mut self) {
        self.message_counts.clear();
    }
    
    /// Get status response
    fn get_status_response(&self) -> StatusResponse {
        // Convert message counts to simplified format
        let channel_stats: Vec<(String, u64)> = self.message_counts
            .iter()
            .map(|(k, v)| (format!("{:?}", k), *v as u64))
            .collect();

        StatusResponse {
            client_count: self.connected_clients.len() as u64,
            message_count: self.message_history.len() as u64,
            channel_stats,
        }
    }
    
    /// Get history response
    fn get_history_response(&self) -> HistoryResponse {
        HistoryResponse {
            entries: self.message_history.clone(),
        }
    }
}

#[hyperprocess(
    name = "Message Log App",
    ui = Some(HttpBindingConfig::default()),
    endpoints = vec![
        Binding::Http { 
            path: "/api/status", 
            config: HttpBindingConfig::new(false, false, false, None) 
        },
        Binding::Http { 
            path: "/api/history", 
            config: HttpBindingConfig::new(false, false, false, None) 
        },
        Binding::Http { 
            path: "/api/clear-history", 
            config: HttpBindingConfig::new(false, false, false, None) 
        },
        Binding::Ws { 
            path: "/", 
            config: WsBindingConfig::default() 
        }
    ],
    save_config = SaveOptions::EveryMessage,
    wit_world = "message-log-app-dot-os-v0"
)]
impl AppState {
    #[init]
    fn initialize(&mut self) {
        // Initialize logging
        init_logging(Level::DEBUG, Level::INFO, None, None, None).unwrap();
        info!("Message Log App initialized");
        
        // Set default configuration
        self.config = AppConfig {
            max_history: 100,
            log_content: true,
        };
        
        // Log initialization
        log_message(
            self,
            "System".to_string(),
            MessageChannel::Internal,
            MessageType::Other("Initialization".to_string()),
            Some("Application started".to_string()),
        );
    }
    
    // HTTP Endpoints with explicit return types
    
    #[http]
    fn get_status(&mut self) -> StatusResponse {
        log_message(
            self,
            "HTTP:GET".to_string(),
            MessageChannel::HttpApi,
            MessageType::HttpGet,
            Some("Status request".to_string()),
        );
        
        self.get_status_response()
    }
    
    #[http]
    fn get_history(&mut self) -> HistoryResponse {
        log_message(
            self,
            "HTTP:GET".to_string(),
            MessageChannel::HttpApi,
            MessageType::HttpGet,
            Some("History request".to_string()),
        );
        
        self.get_history_response()
    }
    
    #[http]
    fn clear_history(&mut self) -> SuccessResponse {
        // Clear the history
        self.message_history.clear();
        self.clear_counts();
        
        log_message(
            self,
            "HTTP:POST".to_string(),
            MessageChannel::HttpApi,
            MessageType::HttpPost,
            Some("History cleared".to_string()),
        );
        
        SuccessResponse {
            success: true,
            message: "History cleared successfully".to_string(),
        }
    }
    
    #[http]
    fn log_custom_message(&mut self, message_type: String, content: String) -> SuccessResponse {
        // Log a custom message
        log_message(
            self,
            "HTTP:Custom".to_string(),
            MessageChannel::HttpApi,
            MessageType::Other(message_type),
            Some(content),
        );
        
        SuccessResponse {
            success: true,
            message: "Custom message logged successfully".to_string(),
        }
    }
    
    // WebSocket handling
    
    #[ws]
    fn handle_websocket(&mut self, channel_id: u32, message_type: WsMessageType, blob: LazyLoadBlob) {
        // Handle WebSocket messages
        if let Ok(message_str) = std::str::from_utf8(&blob.bytes()) {
            // Try to parse as JSON
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(message_str) {
                // Check for various commands
                if let Some(command) = json_value.get("command").and_then(|c| c.as_str()) {
                    match command {
                        "get_status" => {
                            log_message(
                                self,
                                "WebSocket:GetStatus".to_string(),
                                MessageChannel::Websocket,
                                MessageType::WebsocketPushA,
                                Some("Status requested".to_string()),
                            );
                            
                            let status = self.get_status_response();
                            
                            // Send status as JSON response
                            if let Ok(response_json) = serde_json::to_string(&status) {
                                info!("Sending status to client {}", channel_id);
                                send_ws_push(
                                    channel_id,
                                    WsMessageType::Text,
                                    LazyLoadBlob {
                                        mime: Some("application/json".to_string()),
                                        bytes: response_json.as_bytes().to_vec(),
                                    },
                                );
                            }
                        },
                        "get_history" => {
                            log_message(
                                self,
                                "WebSocket:GetHistory".to_string(),
                                MessageChannel::Websocket,
                                MessageType::WebsocketPushA,
                                Some("History requested".to_string()),
                            );
                            
                            let history = self.get_history_response();
                            
                            // Send history as JSON response
                            if let Ok(response_json) = serde_json::to_string(&history) {
                                info!("Sending history to client {}", channel_id);
                                send_ws_push(
                                    channel_id,
                                    WsMessageType::Text,
                                    LazyLoadBlob {
                                        mime: Some("application/json".to_string()),
                                        bytes: response_json.as_bytes().to_vec(),
                                    },
                                );
                            }
                        },
                        "clear_history" => {
                            // Clear the history
                            self.message_history.clear();
                            self.clear_counts();
                            
                            log_message(
                                self,
                                "WebSocket:Clear".to_string(),
                                MessageChannel::Websocket,
                                MessageType::WebsocketPushA,
                                Some("History cleared".to_string()),
                            );
                            
                            let response = SuccessResponse {
                                success: true,
                                message: "History cleared successfully".to_string(),
                            };
                            
                            // Send confirmation as JSON response
                            if let Ok(response_json) = serde_json::to_string(&response) {
                                info!("Sending clear confirmation to client {}", channel_id);
                                send_ws_push(
                                    channel_id,
                                    WsMessageType::Text,
                                    LazyLoadBlob {
                                        mime: Some("application/json".to_string()),
                                        bytes: response_json.as_bytes().to_vec(),
                                    },
                                );
                            }
                        },
                        "log_message" => {
                            if let (Some(msg_type), Some(msg_content)) = (
                                json_value.get("message_type").and_then(|t| t.as_str()),
                                json_value.get("content").and_then(|c| c.as_str())
                            ) {
                                log_message(
                                    self,
                                    "WebSocket:Custom".to_string(),
                                    MessageChannel::Websocket,
                                    MessageType::WebsocketPushB,
                                    Some(format!("Type: {}, Content: {}", msg_type, msg_content)),
                                );
                                
                                let response = SuccessResponse {
                                    success: true,
                                    message: "Custom message logged successfully".to_string(),
                                };
                                
                                // Send confirmation as JSON response
                                if let Ok(response_json) = serde_json::to_string(&response) {
                                    info!("Sending log confirmation to client {}", channel_id);
                                    send_ws_push(
                                        channel_id,
                                        WsMessageType::Text,
                                        LazyLoadBlob {
                                            mime: Some("application/json".to_string()),
                                            bytes: response_json.as_bytes().to_vec(),
                                        },
                                    );
                                }
                            }
                        },
                        _ => {
                            // Unknown command
                            let error = ErrorResponse {
                                success: false,
                                code: 400,
                                message: format!("Unknown command: {}", command),
                            };
                            
                            if let Ok(error_json) = serde_json::to_string(&error) {
                                send_ws_push(
                                    channel_id,
                                    WsMessageType::Text,
                                    LazyLoadBlob {
                                        mime: Some("application/json".to_string()),
                                        bytes: error_json.as_bytes().to_vec(),
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    
    #[remote]
    fn external_get_status(&mut self) -> StatusResponse {
        log_message(
            self,
            "External:GetStatus".to_string(),
            MessageChannel::External,
            MessageType::ResponseReceived,
            Some("Status requested externally".to_string()),
        );
        
        self.get_status_response()
    }
    
    #[remote]
    fn external_get_history(&mut self) -> HistoryResponse {
        log_message(
            self,
            "External:GetHistory".to_string(),
            MessageChannel::External,
            MessageType::ResponseReceived,
            Some("History requested externally".to_string()),
        );
        
        self.get_history_response()
    }
    
    #[remote]
    fn external_clear_history(&mut self) -> SuccessResponse {
        // Clear the history
        self.message_history.clear();
        self.clear_counts();
        
        log_message(
            self,
            "External:ClearHistory".to_string(),
            MessageChannel::External,
            MessageType::ResponseReceived,
            Some("History cleared externally".to_string()),
        );
        
        SuccessResponse {
            success: true,
            message: "History cleared successfully".to_string(),
        }
    }
    
    #[local]
    #[remote]
    fn log_external_message(&mut self, message_type: String, content: String) -> SuccessResponse {
        // Get the source address
        let source_address = self.get_source().to_string();
        
        log_message(
            self,
            format!("External:{}", source_address),
            MessageChannel::External,
            MessageType::Other(message_type),
            Some(content),
        );
        
        SuccessResponse {
            success: true,
            message: "Message logged successfully".to_string(),
        }
    }
    
    #[timer]
    fn handle_timer(&mut self) {
        // Log the timer message
        log_message(
            self,
            "Timer".to_string(),
            MessageChannel::Timer,
            MessageType::TimerTick,
            Some("Timer event received".to_string()),
        );
        info!("Received timer message");
        
        // Send status updates to all connected websocket clients
        let status = self.get_status_response();
        
        if let Ok(status_json) = serde_json::to_string(&status) {
            for (client_id, _) in &self.connected_clients {
                // Sending WS push to all connected clients
                send_ws_push(
                    *client_id,
                    WsMessageType::Text,
                    LazyLoadBlob {
                        mime: Some("application/json".to_string()),
                        bytes: status_json.as_bytes().to_vec(),
                    },
                );
            }
        }
    }
}

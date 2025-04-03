use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize)]
pub struct StatusResponse {
    pub client_count: u64,
    pub message_count: u64,
    pub channel_stats: Vec<(String, u64)>,
}

#[derive(Serialize, Deserialize)]
pub struct HistoryResponse {
    pub entries: Vec<LogEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct LogEntry {
    pub source: String,
    pub channel: String, // Simplified from MessageChannel
    pub type_name: String, // Simplified from MessageType
    pub content: Option<String>,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageChannel {
    Websocket,
    HttpApi,
    Internal,
    External,
    Timer,
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    WebsocketOpen,
    WebsocketClose,
    WebsocketPushA,
    WebsocketPushB,
    HttpGet,
    HttpPost,
    TimerTick,
    LocalRequest,
    RemoteRequest,
    ResponseReceived,
    TerminalCommand,
    Other(String),
}

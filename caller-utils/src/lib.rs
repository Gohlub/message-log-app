wit_bindgen::generate!({
    path: "target/wit",
    world: "types-message-log-app-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

/// Generated caller utilities for RPC function stubs

pub use hyperware_app_common::SendResult;
pub use hyperware_app_common::send;
use hyperware_process_lib::Address;
use serde_json::json;

/// Generated RPC stubs for the app interface
pub mod app {
    use crate::*;

    /// Generated stub for `get-status` http RPC call
    pub async fn get_status_http_rpc(_target: &str) -> SendResult<StatusResponse> {
        // TODO: Implement HTTP endpoint
        SendResult::Success(StatusResponse::default())
    }
    
    /// Generated stub for `get-history` http RPC call
    pub async fn get_history_http_rpc(_target: &str) -> SendResult<HistoryResponse> {
        // TODO: Implement HTTP endpoint
        SendResult::Success(HistoryResponse::default())
    }
    
    /// Generated stub for `clear-history` http RPC call
    pub async fn clear_history_http_rpc(_target: &str) -> SendResult<SuccessResponse> {
        // TODO: Implement HTTP endpoint
        SendResult::Success(SuccessResponse::default())
    }
    
    /// Generated stub for `log-custom-message` http RPC call
    pub async fn log_custom_message_http_rpc(_target: &str, _message_type:  String, _content:  String) -> SendResult<SuccessResponse> {
        // TODO: Implement HTTP endpoint
        SendResult::Success(SuccessResponse::default())
    }
    
    /// Generated stub for `external-get-status` remote RPC call
    pub async fn external_get_status_remote_rpc(target: &Address) -> SendResult<StatusResponse> {
        let request = json!({"ExternalGetStatus" : {}});
        send::<StatusResponse>(&request, target, 30).await
    }
    
    /// Generated stub for `external-get-history` remote RPC call
    pub async fn external_get_history_remote_rpc(target: &Address) -> SendResult<HistoryResponse> {
        let request = json!({"ExternalGetHistory" : {}});
        send::<HistoryResponse>(&request, target, 30).await
    }
    
    /// Generated stub for `external-clear-history` remote RPC call
    pub async fn external_clear_history_remote_rpc(target: &Address) -> SendResult<SuccessResponse> {
        let request = json!({"ExternalClearHistory" : {}});
        send::<SuccessResponse>(&request, target, 30).await
    }
    
    /// Generated stub for `log-external-message` remote RPC call
    pub async fn log_external_message_remote_rpc(target: &Address, message_type: String, content: String) -> SendResult<SuccessResponse> {
        let request = json!({"LogExternalMessage": (message_type, content)});
        send::<SuccessResponse>(&request, target, 30).await
    }
    
    /// Generated stub for `log-external-message` local RPC call
    pub async fn log_external_message_local_rpc(target: &Address, message_type: String, content: String) -> SendResult<SuccessResponse> {
        let request = json!({"LogExternalMessage": (message_type, content)});
        send::<SuccessResponse>(&request, target, 30).await
    }
    
    
}


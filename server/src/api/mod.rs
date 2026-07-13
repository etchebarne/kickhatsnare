mod audio;
mod library;
mod system;
mod workspace;

use kickhatsnare_core::Core;
use kickhatsnare_protocol::ErrorCode;
use serde_json::Value;

pub fn dispatch(method: &str, params: &Value, core: &mut Core) -> Result<Value, ApiError> {
    let Some((domain, action)) = method.split_once('.') else {
        return Err(ApiError::new(
            ErrorCode::InvalidRequest,
            "method must use the <domain>.<action> format",
        ));
    };

    match domain {
        "audio" => audio::dispatch(method, action, params, core),
        "library" => library::dispatch(method, action, params, core.library()),
        "system" => system::dispatch(method, action, params, core.system()),
        "workspace" => workspace::dispatch(method, action, params, core),
        _ => Err(ApiError::new(
            ErrorCode::UnknownDomain,
            format!("unknown endpoint domain: {domain}"),
        )),
    }
}

pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
}

impl ApiError {
    fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    fn method_not_found(domain: &str, action: &str) -> Self {
        Self::new(
            ErrorCode::MethodNotFound,
            format!("unknown {domain} method: {action}"),
        )
    }
}

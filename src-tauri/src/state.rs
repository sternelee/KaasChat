use goose_extension::Capabilities;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared application state
#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub capabilities: Arc<Mutex<Capabilities>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            capabilities: Arc::new(Mutex::new(Capabilities::new())),
        }
    }
}

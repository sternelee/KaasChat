use goose_extension::Capabilities;

/// Shared application state
#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub capabilities: Capabilities,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            capabilities: Capabilities::new(),
        }
    }
}

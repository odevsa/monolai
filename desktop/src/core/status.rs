use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running {
        url: String,
        port: u16,
        started_at: Instant,
    },
    Stopping,
    Error(String),
    UnexpectedExit {
        exit_code: Option<i32>,
    },
}

#[allow(dead_code)]
impl ServerStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, ServerStatus::Running { .. })
    }

    pub fn is_transitioning(&self) -> bool {
        matches!(self, ServerStatus::Starting | ServerStatus::Stopping)
    }

    pub fn port(&self) -> Option<u16> {
        match self {
            ServerStatus::Running { port, .. } => Some(*port),
            _ => None,
        }
    }
}

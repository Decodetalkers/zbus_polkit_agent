#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("zbus error")]
    Zbus(#[from] zbus::Error),
    #[error("error during systemd invoke")]
    System(#[from] systemd::Error),
    #[error("Session Type Unknown: {0}")]
    SessionUnknown(String),
    #[error("Session Unmatch")]
    SessionUnmatch,
    #[error("Server did not provided important information")]
    SessionInnerError
}

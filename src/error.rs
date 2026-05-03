#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("zbus error")]
    Zbus(#[from] zbus::Error),
    #[error("error during systemd invoke")]
    System(#[from] systemd::Error),
}

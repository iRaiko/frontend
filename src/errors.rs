use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error
{
    #[error(transparent)]
    Parse(#[from] serde_json::Error),
    #[error(transparent)]
    ServerNetwork(#[from] reqwest::Error),
}
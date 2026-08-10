use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChurroError {
    #[error(transparent)]
    Connect(#[from] connectrpc::ConnectError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    InvalidUri(#[from] http::uri::InvalidUri),
    #[error(transparent)]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("error receiving data on websocket: {0}")]
    WebsocketRecv(String),
    #[error("{0} resource not found")]
    ResourceNotFound(&'static str),
}

pub type CResult<T = ()> = Result<T, ChurroError>;

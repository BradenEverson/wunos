use warp::filters::ws::Message;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Tokio Send Error: {0}")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<Message>),
}

pub type Result<T> = std::result::Result<T, ServerError>;

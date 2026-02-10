use thiserror::Error;



#[derive(Error, Debug)]
pub enum serdeError {

    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("{0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error("{0}")]
    decryptError(String),

    #[error("{0}")]
    decodeError(#[from] base64::DecodeError),
}
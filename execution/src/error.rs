use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use eyre::Report;
// use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum MainProcessError {
    #[error(transparent)]
    Axum(#[from] axum::http::Error),

    #[error(transparent)]
    Unexpected(#[from] Report),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Invalid transcript proof: {0}")]
    BadTranscriptProof(String),

    #[error("Invalid content schema: {0}")]
    BadContentSchema(String),
}

impl MainProcessError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Axum(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BadRequest(_) | Self::BadTranscriptProof(_) | Self::BadContentSchema(_) => {
                StatusCode::BAD_REQUEST
            }
        }
    }
}

impl IntoResponse for MainProcessError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.to_string();
        (status, message).into_response()
    }
}

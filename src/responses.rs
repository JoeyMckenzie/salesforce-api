use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TransactionSuccessfulResponse {
    message: String,
    #[serde(skip_serializing)]
    status: StatusCode,
}

impl TransactionSuccessfulResponse {
    pub fn new(message: String, status: StatusCode) -> Self {
        Self { message, status }
    }
}

impl IntoResponse for TransactionSuccessfulResponse {
    fn into_response(self) -> Response {
        (self.status, Json(self)).into_response()
    }
}

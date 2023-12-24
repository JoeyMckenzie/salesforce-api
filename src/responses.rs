use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TransactionSuccessfulResponse {
    message: String
}

impl TransactionSuccessfulResponse {
    pub fn new(message: String) -> Self {
        Self {
            message
        }
    }
}

impl IntoResponse for TransactionSuccessfulResponse {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, Json(self)).into_response()
    }
}
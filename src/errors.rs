//! BubbleHearth errors that can occur during at any point
//! during the request cycle to Blizzard, mappings, builders, etc.

use aws_sdk_ssm::error::SdkError;
use aws_sdk_ssm::operation::get_parameter::GetParameterError;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;

/// Wrapped result type useful for marshalling between library and dependencies errors.
pub type ServiceResult<T> = Result<T, ServiceError>;

/// Errors that can occur within the client, including mapped errors from reqwest.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Represents any reqwest that has failed, propagating the error context.
    #[error("{0}")]
    ClientRequestFailed(#[from] reqwest::Error),
    /// Represents a generic error when attempting to retrieve configuration from SSM.
    #[error("The parameter name was not found.")]
    ParameterConfigurationNameEmpty,
    /// Represents a generic error when attempting to retrieve configuration from SSM.
    #[error("{0}")]
    ParameterConfigurationFailedToLoad(#[from] SdkError<GetParameterError>),
    /// Represents an invalid empty configuration error.
    #[error("Parameter configuration {0} is empty.")]
    ParameterConfigurationEmpty(String),
    /// Represents a failure when loading application configuration from SSM at startup.
    #[error("{0}")]
    LoadConfigurationFailed(#[from] tokio::task::JoinError),
    /// Represents a failure when loading application configuration from SSM at startup.
    #[error("{0}")]
    ConfigurationDeserializationFailed(#[from] serde_json::Error),
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::ParameterConfigurationNameEmpty => (StatusCode::INTERNAL_SERVER_ERROR, Self::ParameterConfigurationNameEmpty.to_string()),
            Self::ParameterConfigurationFailedToLoad(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            Self::LoadConfigurationFailed(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            Self::ConfigurationDeserializationFailed(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("Unexpected error occurred."),
            ),
        };

        let body = json!({
            "message": error_message
        });

        (status, Json(body)).into_response()
    }
}
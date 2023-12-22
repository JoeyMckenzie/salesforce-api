//! BubbleHearth errors that can occur during at any point
//! during the request cycle to Blizzard, mappings, builders, etc.

use aws_sdk_ssm::error::SdkError;
use aws_sdk_ssm::operation::get_parameter::GetParameterError;
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

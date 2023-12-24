#![forbid(unsafe_code)]
#![warn(
    dead_code,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    unused_allocation,
    trivial_numeric_casts,
    clippy::single_char_pattern
)]

mod config;
mod errors;
mod salesforce;
mod router;
mod requests;
mod responses;
mod extractors;

use aws_sdk_ssm::types::Parameter;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{SalesforceConfiguration, ServiceConfiguration};
use crate::errors::{ServiceError, ServiceResult};
use crate::router::ServiceRouter;

const SALESFORCE_PARAMETER: &str = "/globals/salesforce/uw";

const SERVICE_PARAMETER: &str = "/services/DLPEvent/configSection/App";

#[tokio::main]
async fn main() -> ServiceResult<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "salesforce_api=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Application initialized, loading API configuration");

    let aws_config = aws_config::load_from_env().await;
    let client = aws_sdk_ssm::Client::new(&aws_config);

    let task_load_salesforce_config =
        tokio::spawn(load_configuration(client.clone(), SALESFORCE_PARAMETER));
    let task_load_service_config = tokio::spawn(load_configuration(client, SERVICE_PARAMETER));

    let (load_salesforce_config_result, load_service_config_result) =
        tokio::try_join!(task_load_salesforce_config, task_load_service_config,)?;

    info!("Service configuration loaded from SSM, parsing values");

    let salesforce_config: SalesforceConfiguration = load_salesforce_config_result?.try_into()?;
    let service_config: ServiceConfiguration = load_service_config_result?.try_into()?;
    let port = service_config.port.unwrap_or(8080);
    let router = ServiceRouter::new_router(salesforce_config, service_config);

    info!("Configuration successfully parsed, starting server on port {}", port);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    axum::serve(listener, router).await.expect("Failed to start API server.");

    Ok(())
}

#[tracing::instrument(ret, skip(client))]
async fn load_configuration(
    client: aws_sdk_ssm::Client,
    parameter_path: &str,
) -> Result<Parameter, ServiceError> {
    client
        .get_parameter()
        .name(parameter_path)
        .with_decryption(true)
        .send()
        .await?
        .parameter
        .ok_or(ServiceError::ParameterConfigurationEmpty(
            parameter_path.into(),
        ))
}

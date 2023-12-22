use aws_sdk_ssm::types::Parameter;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use salesforce::config::{SalesforceConfiguration, ServiceConfiguration};
use salesforce::errors::{ServiceError, ServiceResult};

const SALESFORCE_PARAMETER: &str = "/globals/salesforce/uw";

const SERVICE_PARAMETER: &str = "/services/DLPEvent/configSection/App";

#[tokio::main]
async fn main() -> ServiceResult<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "service=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Application initialized, loading API configuration");

    let aws_config = aws_config::load_from_env().await;
    let client = aws_sdk_ssm::Client::new(&aws_config);

    let task_load_salesforce_config =
        tokio::spawn(load_configuration(client.clone(), SALESFORCE_PARAMETER));
    let task_load_service_config = tokio::spawn(load_configuration(client, SERVICE_PARAMETER));

    // Wait for both tasks to complete
    let (load_salesforce_config_result, load_service_config_result) =
        tokio::try_join!(task_load_salesforce_config, task_load_service_config,)?;

    let salesforce_config: SalesforceConfiguration = load_salesforce_config_result?.try_into()?;
    let service_config: ServiceConfiguration = load_service_config_result?.try_into()?;

    dbg!(salesforce_config);
    dbg!(service_config);

    info!("Service configuration loaded, parsing into configuration structs");

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

use aws_sdk_ssm::types::Parameter;
use serde::Deserialize;
use tracing::info;

use crate::errors::{ServiceError, ServiceResult};

const UW_SALESFORCE_PARAMETER: &str = "/globals/salesforce/uw";

const NF_SALESFORCE_PARAMETER: &str = "/globals/salesforce/NationalFunding.Gen1/nfautomationuser";

const QB_SALESFORCE_PARAMETER: &str = "/globals/salesforce.qb.qbautomationuser";

const SERVICE_PARAMETER: &str = "/services/DLPEvent/configSection/App";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesforceConfiguration {
    #[serde(rename = "salesForceUrl")]
    pub salesforce_url: String,
    pub user_name: String,
    pub password: String,
    pub consumer_key: String,
    pub consumer_secret: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceConfiguration {
    pub encryption_base_uri: String,
    pub timeout_seconds: Option<u64>,
    pub port: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct AggregateSystemConfiguration {
    pub uw_salesforce_config: SalesforceConfiguration,
    pub nf_salesforce_config: SalesforceConfiguration,
    pub qb_salesforce_config: SalesforceConfiguration,
    pub service_config: ServiceConfiguration,
}

impl TryFrom<Parameter> for ServiceConfiguration {
    type Error = ServiceError;

    fn try_from(parameter: Parameter) -> Result<Self, Self::Error> {
        resolve_config_from_json::<Self>(&parameter)
    }
}

impl TryFrom<Parameter> for SalesforceConfiguration {
    type Error = ServiceError;

    fn try_from(parameter: Parameter) -> Result<Self, Self::Error> {
        resolve_config_from_json::<Self>(&parameter)
    }
}

fn resolve_config_from_json<'a, T>(parameter: &'a Parameter) -> Result<T, ServiceError>
where
    T: serde::Deserialize<'a>,
{
    match &parameter.value {
        None => match &parameter.name {
            None => Err(ServiceError::ParameterConfigurationNameEmpty),
            Some(name) => Err(ServiceError::ParameterConfigurationEmpty(name.to_owned())),
        },
        Some(value) => {
            let config: T = serde_json::from_str(value)?;
            Ok(config)
        }
    }
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

pub async fn load_salesforce_configurations() -> ServiceResult<AggregateSystemConfiguration> {
    let aws_config = aws_config::load_from_env().await;
    let client = aws_sdk_ssm::Client::new(&aws_config);

    let task_load_uw_salesforce_config =
        tokio::spawn(load_configuration(client.clone(), UW_SALESFORCE_PARAMETER));
    let task_load_nf_salesforce_config =
        tokio::spawn(load_configuration(client.clone(), NF_SALESFORCE_PARAMETER));
    let task_load_qb_salesforce_config =
        tokio::spawn(load_configuration(client.clone(), QB_SALESFORCE_PARAMETER));
    let task_load_service_config = tokio::spawn(load_configuration(client, SERVICE_PARAMETER));

    let (
        load_uw_salesforce_config_result,
        load_nf_salesforce_config_result,
        load_qb_salesforce_config_result,
        load_service_config_result,
    ) = tokio::try_join!(
        task_load_uw_salesforce_config,
        task_load_nf_salesforce_config,
        task_load_qb_salesforce_config,
        task_load_service_config,
    )?;

    info!("Service configuration loaded from SSM, parsing values");

    let uw_salesforce_config: SalesforceConfiguration =
        load_uw_salesforce_config_result?.try_into()?;
    let nf_salesforce_config: SalesforceConfiguration =
        load_nf_salesforce_config_result?.try_into()?;
    let qb_salesforce_config: SalesforceConfiguration =
        load_qb_salesforce_config_result?.try_into()?;
    let service_config: ServiceConfiguration = load_service_config_result?.try_into()?;

    Ok(AggregateSystemConfiguration {
        uw_salesforce_config,
        nf_salesforce_config,
        qb_salesforce_config,
        service_config,
    })
}

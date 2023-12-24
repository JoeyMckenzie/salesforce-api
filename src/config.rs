use aws_sdk_ssm::types::Parameter;
use serde::Deserialize;

use crate::errors::ServiceError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesforceConfiguration {
    #[serde(rename = "salesForceUrl")]
    salesforce_url: String,
    user_name: String,
    password: String,
    consumer_key: String,
    consumer_secret: String,
    soap_endpoint: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceConfiguration {
    pub encryption_base_uri: String,
    pub timeout_seconds: Option<u64>,
    pub port: Option<u16>
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

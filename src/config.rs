use aws_sdk_ssm::types::Parameter;
use serde::Deserialize;

use crate::errors::ServiceError;

#[derive(Debug, Deserialize)]
pub struct SalesforceConfiguration {
    #[serde(rename = "salesForceUrl")]
    salesforce_url: String,
    #[serde(rename = "userName")]
    user_name: String,
    password: String,
    #[serde(rename = "consumerKey")]
    consumer_key: String,
    #[serde(rename = "consumerSecret")]
    consumer_secret: String,
    #[serde(rename = "soapEndpoint")]
    soap_endpoint: String,
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfiguration {
    #[serde(rename = "EncryptionBaseUri")]
    encryption_url: String,
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

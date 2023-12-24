use std::time::Duration;
use crate::config::{SalesforceConfiguration, ServiceConfiguration};

#[derive(Debug)]
pub struct SalesforceService {
    client: reqwest::Client,
    config: SalesforceConfiguration
}

impl SalesforceService {
    pub fn new(salesforce_configuration: SalesforceConfiguration, service_configuration: ServiceConfiguration) -> Self {
        let timeout_duration = Duration::from_secs(service_configuration.timeout_seconds.unwrap_or(5));
        let client = reqwest::ClientBuilder::new()
            .timeout(timeout_duration)
            .build()
            .unwrap();
        Self {
            client,
            config: salesforce_configuration
        }
    }
}
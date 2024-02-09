use std::sync::Arc;

use crate::config::AggregateSystemConfiguration;
use crate::organization::SalesforceOrganization;
use crate::salesforce::service::SalesforceService;

#[derive(Debug)]
pub struct SalesforceServiceResolver {
    uw_service: Arc<SalesforceService>,
    nf_service: Arc<SalesforceService>,
    qb_service: Arc<SalesforceService>,
}

impl SalesforceServiceResolver {
    pub fn new(aggregate_system_configuration: AggregateSystemConfiguration) -> Self {
        let service_configuration = aggregate_system_configuration.service_config;

        Self {
            uw_service: Arc::new(SalesforceService::new(
                aggregate_system_configuration.uw_salesforce_config,
                service_configuration.clone(),
            )),
            nf_service: Arc::new(SalesforceService::new(
                aggregate_system_configuration.nf_salesforce_config,
                service_configuration.clone(),
            )),
            qb_service: Arc::new(SalesforceService::new(
                aggregate_system_configuration.qb_salesforce_config,
                service_configuration,
            )),
        }
    }

    pub fn resolve(&self, organization: SalesforceOrganization) -> Arc<SalesforceService> {
        match organization {
            SalesforceOrganization::NationalFunding => self.nf_service.clone(),
            SalesforceOrganization::QuickBridge => self.qb_service.clone(),
            SalesforceOrganization::Underwriting => self.uw_service.clone(),
        }
    }
}

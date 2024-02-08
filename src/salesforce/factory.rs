use crate::config::AggregateSystemConfiguration;
use crate::organization::SalesforceOrganization;
use crate::salesforce::service::SalesforceService;

#[derive(Debug)]
pub struct SalesforceServiceResolver {
    uw_service: SalesforceService,
    nf_service: SalesforceService,
    qb_service: SalesforceService,
}

impl SalesforceServiceResolver {
    pub fn new(aggregate_system_configuration: AggregateSystemConfiguration) -> Self {
        let service_configuration = aggregate_system_configuration.service_config;

        Self {
            uw_service: SalesforceService::new(
                aggregate_system_configuration.uw_salesforce_config,
                service_configuration.clone(),
            ),
            nf_service: SalesforceService::new(
                aggregate_system_configuration.nf_salesforce_config,
                service_configuration.clone(),
            ),
            qb_service: SalesforceService::new(
                aggregate_system_configuration.qb_salesforce_config,
                service_configuration,
            ),
        }
    }

    pub fn resolve(&self, organization: SalesforceOrganization) -> &SalesforceService {
        match organization {
            SalesforceOrganization::NationalFunding => &self.nf_service,
            SalesforceOrganization::QuickBridge => &self.qb_service,
            SalesforceOrganization::Underwriting => &self.uw_service,
        }
    }
}

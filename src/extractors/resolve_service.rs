use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::HeaderMap;

use crate::errors::{ServiceError, ServiceResult};
use crate::organization::SalesforceOrganization;
use crate::router::AppState;
use crate::salesforce::service::SalesforceService;

#[derive(Debug)]
pub struct ResolveSalesforceService(pub SalesforceService);

#[async_trait]
impl FromRequestParts<Arc<AppState>> for ResolveSalesforceService {
    type Rejection = ServiceError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let headers = HeaderMap::from_request_parts(parts, state).await;

        match headers {
            Err(e) => Err(ServiceError::InvalidOrganization(e.to_string())),
            Ok(extracted_headers) => match extracted_headers.get("SF-Organization") {
                None => Err(ServiceError::InvalidOrganization(
                    "Salesforce organization header was not found.".to_string(),
                )),
                Some(sf_org) => {
                    let org: ServiceResult<SalesforceOrganization> = sf_org.try_into();

                    match org {
                        Ok(parsed_org) => {
                            let state = state.clone();
                            let resolved_service = match parsed_org {
                                SalesforceOrganization::NationalFunding => state.nf_service.clone(),
                                SalesforceOrganization::QuickBridge => state.qb_service.clone(),
                                SalesforceOrganization::Underwriting => state.uw_service.clone()
                            };
                            
                            Ok(ResolveSalesforceService(resolved_service))
                        }
                        Err(e) => Err(ServiceError::InvalidOrganization(e.to_string())),
                    }
                }
            },
        }
    }
}

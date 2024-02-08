use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::HeaderMap;

use crate::errors::{ServiceError, ServiceResult};
use crate::organization::SalesforceOrganization;
use crate::router::RouterState;
use crate::salesforce::service::SalesforceService;

#[derive(Debug)]
pub struct ResolveSalesforceService(pub Arc<SalesforceService>);

#[async_trait]
impl FromRequestParts<Arc<RouterState>> for ResolveSalesforceService {
    type Rejection = ServiceError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<RouterState>,
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
                            let resolved_service = state.resolver.resolve(parsed_org);
                            Ok(ResolveSalesforceService(resolved_service))
                        }
                        Err(e) => Err(ServiceError::InvalidOrganization(e.to_string())),
                    }
                }
            },
        }
    }
}

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::errors::{ServiceError, ServiceResult};
use crate::organization::SalesforceOrganization;

#[derive(Debug, Copy, Clone)]
pub struct ExtractSalesforceOrg(pub SalesforceOrganization);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractSalesforceOrg
where
    S: Send + Sync,
{
    type Rejection = ServiceError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        match parts.headers.get("SF-Organization") {
            None => Err(ServiceError::InvalidOrganization(
                "Salesforce header value was not found.".to_string(),
            )),
            Some(sf_org) => {
                let org: ServiceResult<SalesforceOrganization> = sf_org.try_into();

                match org {
                    Ok(parsed_org) => Ok(ExtractSalesforceOrg(parsed_org)),
                    Err(e) => Err(ServiceError::InvalidOrganization(e.to_string())),
                }
            }
        }
    }
}

use async_trait::async_trait;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::{Json, RequestPartsExt};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use validator::Validate;

use crate::errors::{ServiceError, ServiceResult};
use crate::organization::SalesforceOrganization;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ServiceError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ExtractSalesforceOrg(pub SalesforceOrganization);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractSalesforceOrg
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<Value>);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        if let Some(sf_org) = parts.headers.get("SF-Organization") {
            let org: ServiceResult<SalesforceOrganization> = sf_org.try_into();

            match org {
                Ok(parsed_org) => Ok(ExtractSalesforceOrg(parsed_org)),
                Err(e) => {
                    let response = json!({
                        "message": e.to_string()
                    });

                    Err((StatusCode::BAD_REQUEST, Json(response)))
                }
            }
        } else {
            let response = json!({
                "message": "Salesforce header value was not found."
            });

            Err((StatusCode::BAD_REQUEST, Json(response)))
        }
    }
}

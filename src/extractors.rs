use async_trait::async_trait;
use axum::extract::{FromRequest, FromRequestParts, MatchedPath, Request, RequestParts};
use axum::{BoxError, Json, RequestPartsExt};
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use validator::Validate;
use crate::errors::ServiceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidationExtractor<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidationExtractor<T>
    where
        T: DeserializeOwned + Validate,
        S: Send + Sync,
{
    type Rejection = (StatusCode, ServiceError);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidationExtractor(value))
    }
}
use std::sync::Arc;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde_json::Value;
use tracing::info;

use crate::errors::ServiceResult;
use crate::extractors::extract_org::ExtractSalesforceOrg;
use crate::extractors::resolve_service::ResolveSalesforceServiceFromService;
use crate::extractors::validation::ValidatedJson;
use crate::requests::CreateObjectRecordRequest;
use crate::responses::TransactionSuccessfulResponse;
use crate::salesforce::resolver::SalesforceServiceResolver;

#[derive(Debug)]
pub struct RouterState {
    pub resolver: SalesforceServiceResolver,
}

#[derive(Debug, Clone, Copy)]
pub struct ServiceRouter;

impl ServiceRouter {
    pub fn new_router(resolver: SalesforceServiceResolver) -> Router {
        let state = RouterState { resolver };

        Router::new()
            .route("/objects/:name/:id", get(find))
            .route("/objects/:name/:id", put(update))
            .route("/objects/query", post(query))
            .route("/objects", post(create))
            .with_state(Arc::new(state))
    }
}

#[tracing::instrument]
async fn find(
    ResolveSalesforceServiceFromService(service): ResolveSalesforceServiceFromService,
    ExtractSalesforceOrg(org): ExtractSalesforceOrg,
    Path((name, id)): Path<(String, String)>,
) -> ServiceResult<Json<Value>> {
    info!("Received request to find object {name} by id {id}");

    let object = service.get_object_by_id(name, id.clone()).await?;

    Ok(Json(object))
}

#[tracing::instrument]
async fn query(
    ResolveSalesforceServiceFromService(service): ResolveSalesforceServiceFromService,
    soql: String,
) -> ServiceResult<Json<Value>> {
    info!("Received request for SOQL query");

    let objects = service.get_objects(soql).await?;

    Ok(Json(objects))
}

#[tracing::instrument]
async fn create(
    ResolveSalesforceServiceFromService(service): ResolveSalesforceServiceFromService,
    ValidatedJson(request): ValidatedJson<CreateObjectRecordRequest>,
) -> ServiceResult<TransactionSuccessfulResponse> {
    info!("Received request for query, executing...");
    Ok(TransactionSuccessfulResponse::new(
        "Record successfully created.".to_string(),
        StatusCode::CREATED,
    ))
}

#[tracing::instrument]
async fn update(
    ResolveSalesforceServiceFromService(service): ResolveSalesforceServiceFromService,
    Path((name, id)): Path<(String, String)>,
    Json(request): Json<Value>,
) -> ServiceResult<TransactionSuccessfulResponse> {
    info!("Received request for updating object");

    service.update_object(name, id, request).await?;

    Ok(TransactionSuccessfulResponse::new(
        "Record successfully updated.".to_string(),
        StatusCode::OK,
    ))
}

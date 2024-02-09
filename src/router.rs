use std::sync::Arc;

use axum::extract::Path;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde_json::Value;
use tracing::info;

use crate::errors::ServiceResult;
use crate::extractors::resolve_service::ResolveSalesforceService;
use crate::extractors::validation::ValidatedJson;
use crate::requests::CreateObjectRecordRequest;
use crate::responses::TransactionSuccessfulResponse;
use crate::salesforce::factory::SalesforceServiceResolver;

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
            .route("/objects/query", post(query))
            .route("/objects", post(create))
            .route("/objects", put(update))
            .with_state(Arc::new(state))
    }
}

#[tracing::instrument]
async fn find(
    ResolveSalesforceService(service): ResolveSalesforceService,
    Path((name, id)): Path<(String, String)>,
) -> ServiceResult<Json<Value>> {
    info!("Received request to find object {name} by id {id}");

    let object = service.get_object_by_id(name, id.clone()).await?;

    Ok(Json(object))
}

#[tracing::instrument]
async fn query(
    ResolveSalesforceService(service): ResolveSalesforceService,
    soql: String,
) -> ServiceResult<Json<Value>> {
    info!("Received request for SOQL query");

    let objects = service.get_objects(soql).await?;

    Ok(Json(objects))
}

#[tracing::instrument]
async fn create(
    ResolveSalesforceService(service): ResolveSalesforceService,
    ValidatedJson(request): ValidatedJson<CreateObjectRecordRequest>,
) -> ServiceResult<TransactionSuccessfulResponse> {
    info!("Received request for query, executing...");
    Ok(TransactionSuccessfulResponse::new(
        "Record successfully created.".to_string(),
    ))
}

#[tracing::instrument]
async fn update(
    ResolveSalesforceService(service): ResolveSalesforceService,
    ValidatedJson(request): ValidatedJson<CreateObjectRecordRequest>,
) -> ServiceResult<TransactionSuccessfulResponse> {
    info!("Received request for query, executing...");
    Ok(TransactionSuccessfulResponse::new(
        "Record successfully created.".to_string(),
    ))
}

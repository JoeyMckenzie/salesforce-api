use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde_json::Value;
use tracing::info;

use crate::errors::ServiceResult;
use crate::extractors::extract_org::ExtractSalesforceOrg;
use crate::extractors::resolve_service::ResolveSalesforceService;
use crate::extractors::validation::ValidatedJson;
use crate::requests::CreateObjectRecordRequest;
use crate::responses::TransactionSuccessfulResponse;
use crate::salesforce::factory::SalesforceServiceResolver;
use crate::salesforce::service::SalesforceService;

#[derive(Debug)]
pub struct RouterState {
    pub resolver: SalesforceServiceResolver,
}

#[derive(Debug)]
pub struct AppState {
    pub uw_service: SalesforceService,
    pub nf_service: SalesforceService,
    pub qb_service: SalesforceService,
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

#[tracing::instrument(skip(state))]
async fn find(
    ExtractSalesforceOrg(header): ExtractSalesforceOrg,
    State(state): State<Arc<RouterState>>,
    Path((name, id)): Path<(String, String)>,
) -> ServiceResult<Json<Value>> {
    info!("Received request to find object {name} by id {id}");

    let object = state
        .resolver
        .resolve(header)
        .get_object_by_id(name, id.clone())
        .await?;

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
    ValidatedJson(request): ValidatedJson<CreateObjectRecordRequest>,
) -> ServiceResult<TransactionSuccessfulResponse> {
    info!("Received request for query, executing...");
    Ok(TransactionSuccessfulResponse::new(
        "Record successfully created.".to_string(),
    ))
}

#[tracing::instrument]
async fn update(
    ValidatedJson(request): ValidatedJson<CreateObjectRecordRequest>,
) -> ServiceResult<TransactionSuccessfulResponse> {
    info!("Received request for query, executing...");
    Ok(TransactionSuccessfulResponse::new(
        "Record successfully created.".to_string(),
    ))
}

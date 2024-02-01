use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use tracing::info;

use crate::config::{SalesforceConfiguration, ServiceConfiguration};
use crate::errors::ServiceResult;
use crate::extractors::ValidatedJson;
use crate::requests::CreateObjectRecordRequest;
use crate::responses::TransactionSuccessfulResponse;
use crate::salesforce::SalesforceService;

#[derive(Debug)]
pub struct RouterState {
    service: SalesforceService,
}

pub struct ServiceRouter;

impl ServiceRouter {
    pub fn new_router(
        salesforce_configuration: SalesforceConfiguration,
        service_configuration: ServiceConfiguration,
    ) -> Router {
        let state = RouterState {
            service: SalesforceService::new(salesforce_configuration, service_configuration),
        };

        Router::new()
            .route("/objects/:name/:id", get(find))
            .route("/objects", get(query))
            .route("/objects", post(create))
            .with_state(Arc::new(state))
    }
}

#[tracing::instrument(skip(state))]
async fn find(
    State(state): State<Arc<RouterState>>,
    Path((name, id)): Path<(String, String)>,
) -> ServiceResult<Json<Value>> {
    info!("Received request to find object {name} by id {id}, requesting access token");

    let object = state.service.get_object_by_id(name, id.clone()).await?;

    Ok(Json(object))
}

#[tracing::instrument(skip(state))]
async fn query(State(state): State<Arc<RouterState>>, soql: String) -> ServiceResult<String> {
    info!("Received request for query, executing...");
    Ok(soql)
}

#[tracing::instrument(skip(state))]
#[axum::debug_handler]
async fn create(
    State(state): State<Arc<RouterState>>,
    ValidatedJson(request): ValidatedJson<CreateObjectRecordRequest>,
) -> ServiceResult<TransactionSuccessfulResponse> {
    info!("Received request for query, executing...");
    Ok(TransactionSuccessfulResponse::new(
        "Record successfully created.".to_string(),
    ))
}

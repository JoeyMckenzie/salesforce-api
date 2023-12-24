use std::sync::Arc;
use axum::extract::State;
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::routing::{get, post};
use tracing::{info, trace};
use crate::config::{SalesforceConfiguration, ServiceConfiguration};
use crate::errors::ServiceResult;
use crate::requests::CreateObjectRecordRequest;
use crate::responses::TransactionSuccessfulResponse;
use crate::salesforce::SalesforceService;

#[derive(Debug)]
pub struct RouterState {
    service: SalesforceService
}

pub struct ServiceRouter;

impl ServiceRouter {
    pub fn new_router(salesforce_configuration: SalesforceConfiguration, service_configuration: ServiceConfiguration)-> Router {
        let state = RouterState {
            service: SalesforceService::new(salesforce_configuration, service_configuration),
        };

        Router::new()
            .route("/objects", get(query))
            .route("/objects", post(create))
            .with_state(Arc::new(state))
    }
}

#[tracing::instrument(skip(state))]
async fn query(State(state): State<Arc<RouterState>>, soql: String) -> ServiceResult<String> {
    info!("Received request for query, executing...");
    Ok(soql)
}

#[tracing::instrument(skip(state))]
#[axum::debug_handler]
async fn create(State(state): State<Arc<RouterState>>, Json(request): Json<CreateObjectRecordRequest>) -> ServiceResult<TransactionSuccessfulResponse> {
    info!("Received request for query, executing...");
    Ok(TransactionSuccessfulResponse::new("Record successfully created.".to_string()))
}

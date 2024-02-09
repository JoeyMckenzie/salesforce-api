use std::sync::Arc;

use axum::extract::Path;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde_json::Value;
use tracing::info;

use crate::config::AggregateSystemConfiguration;
use crate::errors::ServiceResult;
use crate::extractors::resolve_service::ResolveSalesforceService;
use crate::extractors::validation::ValidatedJson;
use crate::requests::CreateObjectRecordRequest;
use crate::responses::TransactionSuccessfulResponse;
use crate::salesforce::service::SalesforceService;

#[derive(Debug)]
pub struct AppState {
    pub uw_service: SalesforceService,
    pub nf_service: SalesforceService,
    pub qb_service: SalesforceService,
}

#[derive(Debug, Clone, Copy)]
pub struct ServiceRouter;

impl ServiceRouter {
    pub fn new_router(system_configuration: AggregateSystemConfiguration) -> Router {
        let service_config = system_configuration.service_config;
        let state = AppState {
            uw_service: SalesforceService::new(
                system_configuration.uw_salesforce_config,
                service_config.clone(),
            ),
            qb_service: SalesforceService::new(
                system_configuration.qb_salesforce_config,
                service_config.clone(),
            ),
            nf_service: SalesforceService::new(
                system_configuration.nf_salesforce_config,
                service_config,
            ),
        };

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
    ResolveSalesforceService(mut service): ResolveSalesforceService,
    Path((name, id)): Path<(String, String)>,
) -> ServiceResult<Json<Value>> {
    info!("Received request to find object {name} by id {id}");

    let object = service.get_object_by_id(name, id.clone()).await?;

    Ok(Json(object))
}

#[tracing::instrument]
async fn query(
    ResolveSalesforceService(mut service): ResolveSalesforceService,
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

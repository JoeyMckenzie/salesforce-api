use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use salesforce_api::config::load_salesforce_configurations;
use salesforce_api::errors::ServiceResult;
use salesforce_api::router::ServiceRouter;

#[tokio::main]
async fn main() -> ServiceResult<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "salesforce_api=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Application initialized, loading API configuration");

    let system_configuration = load_salesforce_configurations().await?;
    let port = system_configuration.service_config.port;
    let port = port.unwrap_or(8080);
    let router = ServiceRouter::new_router(system_configuration);

    info!(
        "Configuration successfully parsed, starting server on port {}",
        port
    );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    axum::serve(listener, router)
        .await
        .expect("Failed to start API server.");

    Ok(())
}

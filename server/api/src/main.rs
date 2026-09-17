//! ContextLab API server entry point.

use contextlab_api::{api_address, try_build_router_from_current_env};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    init_tracing();

    let listener = tokio::net::TcpListener::bind(api_address()).await?;
    tracing::info!(address = %listener.local_addr()?, "contextlab api listening");

    axum::serve(listener, try_build_router_from_current_env()?).await?;
    Ok(())
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("contextlab_api=info,tower_http=info"));

    fmt().with_env_filter(env_filter).init();
}

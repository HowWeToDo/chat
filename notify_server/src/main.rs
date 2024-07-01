use anyhow::Result;
use notify_server::{get_router, AppConfig};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{
    filter::LevelFilter, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _,
};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "127.0.0.1:6687";

    let config = AppConfig::load().expect("Failed to load config");
    let app = get_router(config).await?;

    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

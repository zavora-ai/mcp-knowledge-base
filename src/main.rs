mod server;
mod store;

use rmcp::{ServiceExt, transport::stdio};
use server::KbServer;
use store::KbStore;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse().unwrap())).init();
    let store = Arc::new(KbStore::new());
    let server = KbServer { store };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

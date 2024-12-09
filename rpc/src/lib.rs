use sqlx::postgres::{PgPool, PgPoolOptions};
mod aggregation_service;
mod db;
mod relay;
mod tests;
use eyre::Result;
use std::{
    net::TcpListener,
    sync::atomic::{AtomicU16, Ordering},
};
use tonic::transport::Server;
use tracing::error;
use types::aggregation::aggregation_service_server::AggregationServiceServer;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(50052);

#[derive(Debug, Clone)]
pub struct AggregationRpc {
    pub db_pool: PgPool,
}

pub async fn new_db(database_url: &str) -> Result<PgPool> {
    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|e| eyre::eyre!("failed to connect to database: {}", e))?;
    Ok(db_pool)
}

pub async fn start(db_pool: PgPool, addr: String) -> Result<()> {
    let grpc_addr = addr.parse()?;
    let aggregation_rpc = AggregationRpc { db_pool };

    println!("Starting RPC server on {}", addr);

    let aggregation_server = AggregationServiceServer::new(aggregation_rpc.clone())
        .max_decoding_message_size(1024 * 1024 * 1024)
        .max_encoding_message_size(1024 * 1024 * 1024);

    let rpc_server = Server::builder().add_service(aggregation_server);

    tokio::select! {
        result = rpc_server.serve(grpc_addr) => {
            if let Err(e) = result {
                error!("RPC server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Ctrl-C received, shutting down...");
        }
    }

    Ok(())
}

pub async fn start_test_rpc_server(db_pool: PgPool) -> eyre::Result<String> {
    let port = loop {
        let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            break port;
        }
    };
    let addr = format!("127.0.0.1:{}", port);
    let grpc_addr = addr.clone();

    tokio::spawn(async move {
        if let Err(e) = start(db_pool, grpc_addr).await {
            eprintln!("error starting server: {:?}", e);
        }
    });

    // Give the server a moment to start up.
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    Ok(addr)
}

pub async fn start_rpc_server(db_pool: PgPool) -> eyre::Result<String> {
    let port = loop {
        let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            break port;
        }
    };
    let addr = format!("127.0.0.1:{}", port);
    let grpc_addr = addr.clone();
    start(db_pool, grpc_addr).await?;

    Ok(addr)
}

use dotenv::dotenv;
use rpc::{new_db, AggregationRpc};
use sqlx::SqlitePool;
use tonic::transport::Server;
use types::aggregation::aggregation_service_server::AggregationServiceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let db_pool = new_db(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let addr = std::env::var("RPC_GRPC_ADDR").unwrap().parse()?;

    let aggregation_rpc = AggregationRpc { db_pool };

    println!("Starting RPC server on {}", addr);

    Server::builder()
        .add_service(AggregationServiceServer::new(aggregation_rpc.clone()))
        .serve(addr)
        .await?;

    Ok(())
}

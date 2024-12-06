use dotenv::dotenv;
use rpc::{new_db, start_rpc_server};

use tonic::transport::Server;
use types::aggregation::aggregation_service_server::AggregationServiceServer;

const DATABASE_URL: &str = "/Users/umadayal/sp1-aggregation-service/rpc/aggregationdb.sqlite";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let db_pool = new_db(DATABASE_URL).await.unwrap();

    // let addr = std::env::var("RPC_GRPC_ADDR").unwrap().parse()?;

    // let aggregation_rpc = AggregationRpc { db_pool };

    // println!("Starting RPC server on {}", addr);

    // Server::builder()
    //     .add_service(AggregationServiceServer::new(aggregation_rpc.clone()))
    //     .serve(addr)
    //     .await?;

    let addr = start_rpc_server(db_pool).await?;

    Ok(())
}

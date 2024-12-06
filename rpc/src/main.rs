use dotenv::dotenv;
use rpc::{new_db, start_rpc_server};

// const DATABASE_URL: &str = "/Users/umadayal/sp1-aggregation-service/rpc/aggregationdb.sqlite";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let db_pool = new_db(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let addr = start_rpc_server(db_pool).await?;

    println!("RPC server started on {}", addr);

    Ok(())
}

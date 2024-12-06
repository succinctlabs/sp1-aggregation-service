use dotenv::dotenv;
use eyre::Result;
// use rpc::start_rpc_server;
use crate::start_rpc_server;
use sqlx::{sqlite::SqlitePool, Row};
use types::aggregation::{
    aggregation_service_client::AggregationServiceClient, AggregateProofRequest, AggregationStatus,
    GetAggregationStatusRequest, GetBatchRequest, ProcessBatchRequest, ResponseStatus,
    UpdateBatchStatusRequest, WriteMerkleTreeRequest,
};

#[sqlx::test(migrations = "./migrations")]
async fn test_aggregate_proof(db_pool: SqlitePool) -> Result<()> {
    dotenv().ok();
    let rpc_addr = start_rpc_server(db_pool).await?;
    println!("Connecting to RPC server at {}", rpc_addr);
    let mut network_client = AggregationServiceClient::connect(format!("https://{}", rpc_addr))
        .await
        .unwrap()
        .max_decoding_message_size(1024 * 1024 * 1024);
    println!("Connected to RPC server");

    // test proof and vk (Vec<u8>)
    let proof: Vec<u8> = vec![1, 2, 3, 4, 5];
    let vk: Vec<u8> = vec![6, 7, 8, 9, 10];

    let test_request = AggregateProofRequest { proof, vk };
    let test_response = network_client
        .aggregate_proof(test_request)
        .await?
        .into_inner();
    println!("Test response: {:?}", test_response);

    let status = network_client
        .get_aggregation_status(GetAggregationStatusRequest {
            proof_id: test_response.proof_id,
        })
        .await?
        .into_inner();
    assert_eq!(status.status, ResponseStatus::AggregationPending as i32);

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_e2e(db_pool: SqlitePool) -> Result<()> {
    dotenv().ok();
    let rpc_addr = std::env::var("RPC_GRPC_ADDR").unwrap();
    println!("Connecting to RPC server at {}", rpc_addr);
    let mut network_client = AggregationServiceClient::connect(format!("https://{}", rpc_addr))
        .await
        .unwrap()
        .max_decoding_message_size(1024 * 1024 * 1024);
    println!("Connected to RPC server");

    let current_timestamp = chrono::Utc::now().timestamp_millis();

    // generate five requests and then get the aggregated data
    let mut proof_ids = Vec::new();
    for _ in 0..5 {
        // generate a random proof and vk
        let proof: Vec<u8> = (0..5).map(|_| rand::random::<u8>()).collect();
        let vk: Vec<u8> = (0..5).map(|_| rand::random::<u8>()).collect();
        let test_request = AggregateProofRequest {
            proof: proof.clone(),
            vk: vk.clone(),
        };
        let test_response = network_client
            .aggregate_proof(test_request)
            .await?
            .into_inner();
        println!("Test response: {:?}", test_response);
        proof_ids.push(test_response.proof_id);
    }

    let batch_request = GetBatchRequest {
        created_after: Some(current_timestamp as u64),
        batch_size: Some(5),
    };
    let batch_response = network_client.get_batch(batch_request).await?.into_inner();
    // println!("Batch response: {:?}", batch_response);
    let requests = batch_response.proofs;
    assert_eq!(requests.len(), 5);
    let batch_id = batch_response.batch_id;

    let process_batch_request = ProcessBatchRequest {
        proofs: requests,
        batch_id: batch_id.clone(),
    };
    let process_batch_response = network_client
        .process_batch(process_batch_request)
        .await?
        .into_inner();
    let leaves = process_batch_response.leaves;
    println!("Leaves: {:?}", leaves);

    let merkle_tree_request = WriteMerkleTreeRequest {
        tree: leaves,
        batch_id: batch_id.clone(),
    };

    Ok(())
}

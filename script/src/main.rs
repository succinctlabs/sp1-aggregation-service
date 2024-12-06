use chrono::Utc;
use sp1_sdk::{ProverClient, SP1Proof, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};
use types::aggregation::{
    aggregation_service_client::AggregationServiceClient, AggregationStatus, GetBatchRequest,
    ProcessBatchRequest, UpdateBatchStatusRequest, WriteMerkleTreeRequest,
};

const AGGREGATION_ELF: &[u8] =
    include_bytes!("../../programs/aggregation-program/elf/riscv32im-succinct-zkvm-elf");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    sp1_sdk::utils::setup_logger();

    let client = ProverClient::new();
    let (aggregation_pk, aggregation_vk) = client.setup(AGGREGATION_ELF);
    let mut network_client =
        AggregationServiceClient::connect(std::env::var("RPC_GRPC_ADDR").unwrap()).await?;

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1800)); // run every 30 minutes
    loop {
        interval.tick().await;
        let current_timestamp = Utc::now().timestamp_millis();
        println!("Current timestamp: {}", current_timestamp);
        println!("Looping every 30 minutes");

        let mut stdin = SP1Stdin::new();

        // Get a batch of proofs
        let batch_response = network_client
            .get_batch(GetBatchRequest {
                created_after: None,
                batch_size: Some(5),
            })
            .await?
            .into_inner();

        let batch_id = batch_response.batch_id;
        let proofs = batch_response.proofs;

        // Process the batch and get the leaves of the merkle tree
        let process_batch_response = network_client
            .process_batch(ProcessBatchRequest {
                batch_id: batch_id.clone(),
                proofs: proofs.clone(),
            })
            .await?
            .into_inner();

        let leaves = process_batch_response.leaves;

        // Write the merkle tree to the database
        network_client
            .write_merkle_tree(WriteMerkleTreeRequest {
                batch_id: batch_id.clone(),
                tree: leaves,
            })
            .await?
            .into_inner();

        // Prove the batch and get the aggregated proof
        for request in proofs {
            println!("Proof ID: {:?}", request.proof_id);
            let proof_deserialized: SP1ProofWithPublicValues =
                serde_json::from_slice(&request.proof).unwrap();
            let vk_deserialized: SP1VerifyingKey = serde_json::from_slice(&request.vk).unwrap();
            let SP1Proof::Compressed(proof) = proof_deserialized.proof else {
                panic!("Proof is not compressed");
            };
            stdin.write_proof(*proof, vk_deserialized.vk);
        }
        let aggregated_proof = client
            .prove(&aggregation_pk, stdin)
            .run()
            .expect("Proving failed");

        // Verify the aggregated proof
        client
            .verify(&aggregated_proof, &aggregation_vk)
            .expect("Proof verification failed");

        // Update the status of all the proofs in the batch to verified
        network_client
            .update_batch_status(UpdateBatchStatusRequest {
                batch_id,
                status: AggregationStatus::Verified as i32,
            })
            .await?;
    }
}

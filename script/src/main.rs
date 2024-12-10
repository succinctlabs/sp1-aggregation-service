use chrono::Utc;
use sp1_sdk::{
    HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
};
use types::aggregation::{
    aggregation_service_client::AggregationServiceClient, AggregationStatus, GetBatchRequest,
    ProcessBatchRequest, UpdateBatchStatusRequest, VerifyAggregationProofRequest,
    WriteMerkleTreeRequest,
};

const AGGREGATION_ELF: &[u8] =
    include_bytes!("../../programs/aggregation-program/elf/riscv32im-succinct-zkvm-elf");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    sp1_sdk::utils::setup_logger();

    let client = ProverClient::new();
    let (aggregation_pk, aggregation_vk) = client.setup(AGGREGATION_ELF);
    let rpc_addr = std::env::var("RPC_GRPC_ADDR").unwrap();
    println!("Connecting to RPC server at {}", rpc_addr);
    let mut network_client = AggregationServiceClient::connect(format!("https://{}", rpc_addr))
        .await
        .unwrap()
        .max_decoding_message_size(1024 * 1024 * 1024);

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1800)); // run every 30 minutes
    loop {
        interval.tick().await;
        let current_timestamp = Utc::now().timestamp_millis();
        println!("Current timestamp: {}", current_timestamp);
        println!("Looping every 30 minutes");

        let mut stdin = SP1Stdin::new();

        // Get a batch of proofs.
        println!("Getting batch of proofs");
        let batch_response = network_client
            .get_batch(GetBatchRequest {
                created_after: None,
                batch_size: Some(5),
            })
            .await?
            .into_inner();

        let batch_id = batch_response.batch_id;
        let proofs = batch_response.proofs;

        println!("Batch ID: {:?}", batch_id);
        println!("Number of proofs in batch: {}", proofs.len());

        // Process the batch and get the leaves of the merkle tree.
        println!("Processing batch");
        let process_batch_response = network_client
            .process_batch(ProcessBatchRequest {
                batch_id: batch_id.clone(),
                proofs: proofs.clone(),
            })
            .await?
            .into_inner();

        let leaves = process_batch_response.leaves;

        // Write the merkle tree to the database
        println!("Writing merkle tree to database");
        network_client
            .write_merkle_tree(WriteMerkleTreeRequest {
                batch_id: batch_id.clone(),
                tree: leaves,
            })
            .await?
            .into_inner();

        // Prove the batch and get the aggregated proof.
        println!("Proving batch");
        let mut vkeys = Vec::new();
        let mut public_values = Vec::new();
        for request in proofs {
            println!("Proof ID: {:?}", request.proof_id);
            let proof_deserialized: SP1ProofWithPublicValues =
                serde_json::from_slice(&request.proof).unwrap();
            let vk_deserialized: SP1VerifyingKey = serde_json::from_slice(&request.vk).unwrap();

            vkeys.push(vk_deserialized.hash_u32());
            public_values.push(proof_deserialized.public_values.to_vec());

            let SP1Proof::Compressed(proof) = proof_deserialized.proof else {
                panic!("Proof is not compressed");
            };
            stdin.write_proof(*proof, vk_deserialized.vk);
        }
        stdin.write::<Vec<[u32; 8]>>(&vkeys);
        stdin.write::<Vec<Vec<u8>>>(&public_values);

        let aggregated_proof = client
            .prove(&aggregation_pk, stdin)
            .run()
            .expect("Proving failed");

        // Verify the aggregated proof
        println!("Verifying aggregated proof");
        let aggregated_proof_bytes = bincode::serialize(&aggregated_proof).unwrap();
        let response = network_client
            .verify_aggregation_proof(VerifyAggregationProofRequest {
                proof: aggregated_proof_bytes,
                batch_id: batch_id.clone(),
            })
            .await
            .expect("Failed to verify aggregation proof")
            .into_inner();
        println!("tx: {:?}", response.tx_hash);
        // println!("Response: {:?}", response);
        // client
        //     .verify(&aggregated_proof, &aggregation_vk)
        //     .expect("Proof verification failed");

        // Update the status of all the proofs in the batch to verified.
        println!("Updating batch status to verified");
        network_client
            .update_batch_status(UpdateBatchStatusRequest {
                batch_id,
                status: AggregationStatus::Verified as i32,
            })
            .await?;
    }
}

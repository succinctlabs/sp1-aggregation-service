// use sp1_sdk::{AggregationClient, ProverClient, SP1Stdin};
use types::aggregation::{
    aggregation_service_client::AggregationServiceClient, AggregateProofRequest,
};

const FIBONACCI_ELF: &[u8] =
    include_bytes!("../../programs/fibonacci-program/elf/riscv32im-succinct-zkvm-elf");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // println!("Starting");
    // dotenv::dotenv().ok();
    // println!("Loading environment variables");
    // let client = ProverClient::new();
    // let aggregation_client = AggregationClient::new();

    // let (pk, vk) = client.setup(FIBONACCI_ELF);
    // let vk_serialized = bincode::serialize(&vk).unwrap();

    // let mut stdin = SP1Stdin::new();

    // let n = 10;
    // stdin.write(&n);
    // let proof = client
    //     .prove(&pk, stdin)
    //     .compressed()
    //     .run()
    //     .expect("proving failed");

    // let proof_id = aggregation_client.aggregate_proof(proof, vk).await.unwrap();
    // println!("Proof ID: {:?}", proof_id);

    // let status = aggregation_client
    //     .get_aggregation_status(proof_id)
    //     .await
    //     .unwrap();
    // println!("Status: {:?}", status);

    // let merkle_proof = aggregation_client
    //     .get_aggregated_data(proof_id)
    //     .await
    //     .unwrap();
    // println!("Merkle proof: {:?}", merkle_proof);

    Ok(())
}

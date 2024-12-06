use sp1_sdk::{
    include_elf, HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues, SP1Stdin,
    SP1VerifyingKey,
};
use types::aggregation::{
    aggregation_service_client::AggregationServiceClient, AggregateProofRequest,
};

const FIBONACCI_ELF: &[u8] =
    include_bytes!("../../programs/fibonacci-program/elf/riscv32im-succinct-zkvm-elf");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let client = ProverClient::new();
    let mut network = AggregationServiceClient::connect(std::env::var("RPC_GRPC_ADDR").unwrap())
        .await
        .unwrap();
    let (pk, vk) = client.setup(FIBONACCI_ELF);
    let vk_serialized = serde_json::to_vec(&vk).unwrap();

    let n_values = vec![10, 20, 30, 40, 50];
    for n in n_values {
        let mut stdin = SP1Stdin::new();
        stdin.write(&n);
        let proof = client
            .prove(&pk, stdin)
            .compressed()
            .run()
            .expect("proving failed");
        let proof_serialized = serde_json::to_vec(&proof).unwrap();
        let response = network
            .aggregate_proof(AggregateProofRequest {
                proof: proof_serialized,
                vk: vk_serialized.clone(),
            })
            .await?
            .into_inner();
        println!("{:?}", response);
    }
    Ok(())
}

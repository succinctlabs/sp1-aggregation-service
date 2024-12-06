use sp1_sdk::{ProverClient, SP1Stdin};
use types::aggregation::{
    aggregation_service_client::AggregationServiceClient, AggregateProofRequest,
};

const FIBONACCI_ELF: &[u8] =
    include_bytes!("../../programs/fibonacci-program/elf/riscv32im-succinct-zkvm-elf");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting");
    dotenv::dotenv().ok();
    println!("Loading environment variables");
    let client = ProverClient::new();
    let rpc_addr = std::env::var("RPC_GRPC_ADDR").unwrap();
    println!("Connecting to RPC server at {}", rpc_addr);
    let mut network = AggregationServiceClient::connect(format!("https://{}", rpc_addr))
        .await
        .unwrap();
    println!("Connected to RPC server");
    let (pk, vk) = client.setup(FIBONACCI_ELF);
    let vk_serialized = serde_json::to_vec(&vk).unwrap();

    // sp1_sdk::utils::setup_logger();
    let n_values = vec![10, 20, 30, 40, 50];
    for n in n_values {
        println!("Generating proof for {}", n);
        let mut stdin = SP1Stdin::new();
        stdin.write(&n);
        let proof = client
            .prove(&pk, stdin)
            .compressed()
            .run()
            .expect("proving failed");
        let proof_serialized = serde_json::to_vec(&proof).unwrap();
        println!("Sending proof request to RPC server");
        let response = network
            .aggregate_proof(AggregateProofRequest {
                proof: proof_serialized,
                vk: vk_serialized.clone(),
            })
            .await?
            .into_inner();
        println!("Proof response: {:?}", response);
    }
    Ok(())
}

use std::time::Duration;

use alloy_network::EthereumWallet;
use alloy_provider::ProviderBuilder;
use alloy_rpc_types_eth::TransactionReceipt;
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::sol;
use clap::Parser;
use sp1_sdk::SP1ProofWithPublicValues;

sol!(
    #[sol(rpc)]
    "../contracts/src/SP1AggregationVerifier.sol"
);

pub async fn relay_proof(proof: SP1ProofWithPublicValues) -> Result<Vec<u8>, eyre::Error> {
    dotenv::dotenv().ok();
    let rpc_url = std::env::var("RPC_URL")
        .unwrap()
        .parse()
        .expect("Invalid RPC URL");
    let address = std::env::var("CONTRACT_ADDRESS").unwrap().parse()?;
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")
        .unwrap()
        .parse()
        .expect("Invalid private key");
    let wallet = EthereumWallet::from(signer);
    let client = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc_url);
    let contract = SP1AggregationVerifier::new(address, client);
    let tx =
        contract.verifyAggregationProof(proof.public_values.to_vec().into(), proof.bytes().into());
    let pending_tx = tx.send().await?;
    let receipt = pending_tx
        .with_required_confirmations(1)
        .with_timeout(Some(Duration::from_secs(120)))
        .get_receipt()
        .await?;

    Ok(receipt.transaction_hash.to_vec())
}

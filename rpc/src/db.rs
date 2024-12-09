use chrono::Utc;
use eyre::Result;
use sha2::{Digest, Sha256};
use sp1_sdk::{HashableKey, SP1ProofWithPublicValues, SP1VerifyingKey};
use sqlx::{postgres::PgPool, Row};
use types::aggregation::{AggregationStatus, AggregationStatusResponse, ProofRequest};

pub async fn create_request(
    db_pool: &PgPool,
    proof_id: Vec<u8>,
    proof: Vec<u8>,
    vk: Vec<u8>,
) -> Result<(), sqlx::Error> {
    let pending_status = AggregationStatus::Pending;
    let created_at = Utc::now().timestamp_millis();
    sqlx::query(
        r#"INSERT INTO requests (proof_id, status, proof, vk, batch_id, created_at, tx_hash, chain_id, contract_address) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
    )
    .bind(proof_id)
    .bind(pending_status)
    .bind(proof)
    .bind(vk)
    .bind::<Option<Vec<u8>>>(None)
    .bind(created_at)
    .bind::<Option<Vec<u8>>>(None)
    .bind::<Option<i64>>(None)
    .bind::<Option<Vec<u8>>>(None)
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn get_batch(
    db_pool: &PgPool,
    created_after: u64,
    batch_size: u64,
) -> Result<Vec<ProofRequest>, sqlx::Error> {
    let pending_status = AggregationStatus::Pending as i32;
    let requests: Vec<ProofRequest> = sqlx::query_as::<_, ProofRequest>(
        r#"SELECT proof_id, status, proof, vk, batch_id, created_at
           FROM requests
           WHERE created_at > $1 AND status = $2
           ORDER BY created_at ASC
           LIMIT $3"#,
    )
    .bind(created_after as i64)
    .bind(pending_status)
    .bind(batch_size as i64)
    .fetch_all(db_pool)
    .await
    .expect("Failed to get batch.");
    Ok(requests)
}

pub async fn write_merkle_tree(
    db_pool: &PgPool,
    merkle_tree: Vec<u8>,
    batch_id: Vec<u8>,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"INSERT INTO merkle_trees (batch_id, tree) VALUES ($1, $2)"#)
        .bind(batch_id)
        .bind(merkle_tree)
        .execute(db_pool)
        .await?;
    Ok(())
}
pub async fn get_merkle_tree(db_pool: &PgPool, proof_id: Vec<u8>) -> Result<Vec<u8>, sqlx::Error> {
    let batch_row = sqlx::query(r#"SELECT batch_id FROM requests WHERE proof_id = $1"#)
        .bind(proof_id)
        .fetch_one(db_pool)
        .await?;
    let batch_id = batch_row.get::<&[u8], _>("batch_id").to_vec();
    // let batch_row = sqlx::query

    let tree = sqlx::query(r#"SELECT tree FROM merkle_trees WHERE batch_id = $1"#)
        .bind(batch_id)
        .fetch_one(db_pool)
        .await?;
    Ok(tree.get::<&[u8], _>("tree").to_vec())
}
pub async fn get_leaf(db_pool: &PgPool, proof_id: Vec<u8>) -> Result<Vec<u8>, sqlx::Error> {
    let proof_row = sqlx::query(r#"SELECT proof, vk FROM requests WHERE proof_id = $1"#)
        .bind(proof_id)
        .fetch_one(db_pool)
        .await?;
    let proof_bytes = proof_row.get::<&[u8], _>("proof").to_vec();
    let proof: SP1ProofWithPublicValues = bincode::deserialize(&proof_bytes).unwrap();
    let vk_bytes = proof_row.get::<&[u8], _>("vk").to_vec();
    let vk: SP1VerifyingKey = bincode::deserialize(&vk_bytes).unwrap();
    let public_values = proof.public_values;
    let leaf = Sha256::digest([&vk.hash_bytes(), public_values.as_slice()].concat());
    Ok(leaf.to_vec())
}

pub async fn get_vkey_and_public_values(
    db_pool: &PgPool,
    proof_id: Vec<u8>,
) -> Result<(Vec<u8>, Vec<u8>), sqlx::Error> {
    let proof_row = sqlx::query(r#"SELECT vk, proof FROM requests WHERE proof_id = $1"#)
        .bind(proof_id)
        .fetch_one(db_pool)
        .await?;
    let proof_bytes = proof_row.get::<&[u8], _>("proof").to_vec();
    let proof: SP1ProofWithPublicValues = bincode::deserialize(&proof_bytes).unwrap();
    let vk_bytes = proof_row.get::<&[u8], _>("vk").to_vec();
    let vk: SP1VerifyingKey = bincode::deserialize(&vk_bytes).unwrap();
    let public_values = proof.public_values;
    Ok((vk.hash_bytes().to_vec(), public_values.to_vec()))
}

pub async fn get_proof_status(
    db_pool: &PgPool,
    proof_id: Vec<u8>,
) -> Result<AggregationStatusResponse, sqlx::Error> {
    let proof_row = sqlx::query(r#"SELECT status FROM requests WHERE proof_id = $1"#)
        .bind(proof_id)
        .fetch_one(db_pool)
        .await;

    // Check if the proof_row was found
    match proof_row {
        Ok(row) => {
            let aggregation_status = row.get::<i32, _>("status");
            let response_status = match aggregation_status {
                status if status == AggregationStatus::Pending as i32 => {
                    Ok(AggregationStatusResponse::AggregationPending)
                }
                status if status == AggregationStatus::Aggregated as i32 => {
                    Ok(AggregationStatusResponse::AggregationComplete)
                }
                status if status == AggregationStatus::Verified as i32 => {
                    Ok(AggregationStatusResponse::AggregationVerified)
                }
                _ => Err("Invalid aggregation status"), // Return an error for unexpected status
            }
            .unwrap();
            Ok(response_status)
        }
        Err(_) => {
            // If proof not found, set response status to NotFound
            Ok(AggregationStatusResponse::NotFound)
        }
    }
}

pub async fn process_batch(
    db_pool: &PgPool,
    proofs: Vec<ProofRequest>,
    batch_id: Vec<u8>,
) -> Result<Vec<u8>, sqlx::Error> {
    let mut leaves = Vec::new();
    let aggregated_status = AggregationStatus::Aggregated as i32;
    for request in proofs {
        sqlx::query(r#"UPDATE requests SET status = $1 WHERE proof_id = $2"#)
            .bind(aggregated_status)
            .bind(request.proof_id.clone())
            .execute(db_pool)
            .await?;
        sqlx::query(r#"UPDATE requests SET batch_id = $1 WHERE proof_id = $2"#)
            .bind(batch_id.clone())
            .bind(request.proof_id.clone())
            .execute(db_pool)
            .await?;
        let leaf = get_leaf(db_pool, request.proof_id).await?;
        leaves.push(leaf);
    }

    let leaves_vec = leaves
        .iter()
        .map(|l| l.to_vec())
        .collect::<Vec<Vec<u8>>>()
        .concat();
    Ok(leaves_vec)
}

pub async fn update_batch_status(
    db_pool: &PgPool,
    batch_id: Vec<u8>,
    status: i32,
) -> Result<(), sqlx::Error> {
    // find all proof_ids in batch
    let rows = sqlx::query(r#"SELECT proof_id FROM requests WHERE batch_id = $1"#)
        .bind(batch_id)
        .fetch_all(db_pool)
        .await?;
    let proof_ids = rows
        .iter()
        .map(|r| r.get::<&[u8], _>("proof_id").to_vec())
        .collect::<Vec<Vec<u8>>>();
    // update status for each proof_id
    for proof_id in proof_ids {
        sqlx::query(r#"UPDATE requests SET status = $1 WHERE proof_id = $2"#)
            .bind(status)
            .bind(proof_id)
            .execute(db_pool)
            .await
            .expect("Failed to update requests.");
    }

    Ok(())
}

pub async fn get_tx_context(
    db_pool: &PgPool,
    proof_id: Vec<u8>,
) -> Result<(Vec<u8>, u64, Vec<u8>), sqlx::Error> {
    let tx_row = sqlx::query(
        r#"SELECT tx_hash, chain_id, contract_address FROM requests WHERE proof_id = $1"#,
    )
    .bind(proof_id)
    .fetch_one(db_pool)
    .await?;

    let tx_hash = tx_row.get::<&[u8], _>("tx_hash").to_vec();
    let chain_id = tx_row.get::<i64, _>("chain_id");
    let contract_address = tx_row.get::<&[u8], _>("contract_address").to_vec();
    Ok((tx_hash, chain_id as u64, contract_address))
}

pub async fn update_proof_tx_hash(
    db_pool: &PgPool,
    batch_id: Vec<u8>,
    tx_hash: Vec<u8>,
) -> Result<(), sqlx::Error> {
    let contract_address = std::env::var("CONTRACT_ADDRESS").unwrap();
    let chain_id = std::env::var("CHAIN_ID").unwrap();
    sqlx::query(r#"UPDATE requests SET tx_hash = $1, contract_address = $2, chain_id = $3 WHERE batch_id = $4"#)
        .bind(tx_hash)
        .bind(contract_address)
        .bind(chain_id)
        .bind(batch_id)
        .execute(db_pool)
        .await?;
    Ok(())
}

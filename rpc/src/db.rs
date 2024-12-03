use chrono::Utc;
use eyre::Result;
use sha2::{Digest, Sha256};
use sp1_sdk::{HashableKey, SP1ProofWithPublicValues, SP1VerifyingKey};
use sqlx::{sqlite::SqlitePool, Row};
use std::fs::File;
use std::io::Read;
use types::aggregation::{AggregationStatus, ProofRequest};

pub async fn create_request(
    db_pool: &SqlitePool,
    proof_id: Vec<u8>,
    proof: Vec<u8>,
    vk: Vec<u8>,
) -> Result<(), sqlx::Error> {
    let pending_status = AggregationStatus::Pending;
    let created_at = Utc::now().timestamp_millis();
    sqlx::query(
        r#"INSERT INTO requests (proof_id, proof_uri, vk_uri, status, created_at) VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(proof_id)
    .bind(proof)
    .bind(vk)
    .bind(pending_status)
    .bind(created_at)
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn get_batch(
    db_pool: &SqlitePool,
    created_after: u64,
    batch_size: u64,
) -> Result<Vec<ProofRequest>, sqlx::Error> {
    let request_rows = sqlx::query(r#"SELECT * FROM requests WHERE created_at > $1 AND aggregation_status = 'PENDING' LIMIT $2"#)
        .bind(created_after as i64)
        .bind(batch_size as i64)
        .fetch_all(db_pool)
        .await?;
    let requests = request_rows
        .iter()
        .map(|r| ProofRequest {
            proof_id: r.get::<&[u8], _>("proof_id").to_vec(),
            status: r.get::<i32, _>("status"),
            proof: r.get::<&[u8], _>("proof").to_vec(),
            vk: r.get::<&[u8], _>("vk").to_vec(),
            created_at: r.get::<i64, _>("created_at") as u64,
        })
        .collect();
    Ok(requests)
}

pub async fn write_merkle_tree(
    db_pool: &SqlitePool,
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
pub async fn get_merkle_tree(
    db_pool: &SqlitePool,
    proof_id: Vec<u8>,
) -> Result<Vec<u8>, sqlx::Error> {
    let batch_row = sqlx::query(r#"SELECT batch_id FROM batches WHERE proof_id = $1"#)
        .bind(proof_id)
        .fetch_one(db_pool)
        .await?;
    let batch_id = batch_row.get::<&[u8], _>("batch_id").to_vec();

    let tree = sqlx::query(r#"SELECT tree FROM merkle_trees WHERE batch_id = $1"#)
        .bind(batch_id)
        .fetch_one(db_pool)
        .await?;
    Ok(tree.get::<&[u8], _>("tree").to_vec())
}
pub async fn get_leaf(db_pool: &SqlitePool, proof_id: Vec<u8>) -> Result<Vec<u8>, sqlx::Error> {
    let proof_row = sqlx::query(r#"SELECT proof FROM requests WHERE proof_id = $1"#)
        .bind(proof_id)
        .fetch_one(db_pool)
        .await?;
    let proof_bytes = proof_row.get::<&[u8], _>("proof").to_vec();
    let proof: SP1ProofWithPublicValues = serde_json::from_slice(&proof_bytes).unwrap();
    let vk_bytes = proof_row.get::<&[u8], _>("vk").to_vec();
    let vk: SP1VerifyingKey = serde_json::from_slice(&vk_bytes).unwrap();
    let public_values = proof.public_values;
    let leaf = Sha256::digest([public_values.as_slice(), &vk.hash_bytes()].concat());
    Ok(leaf.to_vec())
}
pub async fn get_proof_status(db_pool: &SqlitePool, proof_id: Vec<u8>) -> Result<i32, sqlx::Error> {
    let proof_row = sqlx::query(r#"SELECT status FROM requests WHERE proof_id = $1"#)
        .bind(proof_id)
        .fetch_one(db_pool)
        .await?;
    Ok(proof_row.get::<i32, _>("status"))
}

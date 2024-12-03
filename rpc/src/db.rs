use chrono::Utc;
use eyre::Result;
use sqlx::{sqlite::SqlitePool, Row};
use types::aggregation::{AggregationStatus, ProofRequest};

pub async fn create_request(
    db_pool: &SqlitePool,
    proof_id: Vec<u8>,
    proof_uri: String,
    vk_uri: String,
) -> Result<(), sqlx::Error> {
    let pending_status = AggregationStatus::Pending;
    let created_at = Utc::now().timestamp_millis();
    sqlx::query(
        r#"INSERT INTO requests (proof_id, proof_uri, vk_uri, status, created_at) VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(proof_id)
    .bind(proof_uri)
    .bind(vk_uri)
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
            proof_uri: r.get::<&str, _>("proof_uri").to_string(),
            vk_uri: r.get::<&str, _>("vk_uri").to_string(),
            created_at: r.get::<i64, _>("created_at") as u64,
        })
        .collect();
    Ok(requests)
}

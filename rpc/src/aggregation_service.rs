use crate::{db, AggregationRpc};
use rand::Rng;
use tonic::{Request, Response, Status};
use types::aggregation::{
    aggregation_service_server::AggregationService, AggregateProofRequest, AggregateProofResponse,
    AggregationStatus, GetAggregatedDataRequest, GetAggregatedDataResponse, GetBatchRequest,
    GetBatchResponse, ProofRequest, ResponseStatus, WriteMerkleProofRequest,
    WriteMerkleProofResponse,
};

#[tonic::async_trait]
impl AggregationService for AggregationRpc {
    async fn get_aggregated_data(
        &self,
        request: Request<GetAggregatedDataRequest>,
    ) -> Result<Response<GetAggregatedDataResponse>, Status> {
        todo!()
    }

    async fn aggregate_proof(
        &self,
        request: Request<AggregateProofRequest>,
    ) -> Result<Response<AggregateProofResponse>, Status> {
        let proof_id: [u8; 32] = rand::thread_rng().gen();
        let req = request.into_inner();
        db::create_request(&self.db_pool, proof_id.to_vec(), req.proof_uri, req.vk_uri)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(AggregateProofResponse {
            proof_id: proof_id.to_vec(),
        }))
    }

    async fn get_batch(
        &self,
        request: Request<GetBatchRequest>,
    ) -> Result<Response<GetBatchResponse>, Status> {
        let req = request.into_inner();
        let batch_id: [u8; 32] = rand::thread_rng().gen();
        let proofs = db::get_batch(
            &self.db_pool,
            req.created_after,
            req.batch_size.unwrap_or(32),
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GetBatchResponse {
            batch_id: batch_id.to_vec(),
            proofs,
        }))
    }

    async fn write_merkle_proof(
        &self,
        request: Request<WriteMerkleProofRequest>,
    ) -> Result<Response<WriteMerkleProofResponse>, Status> {
        todo!()
    }
}

use crate::{db, AggregationRpc};
use rand::Rng;
use tonic::{Request, Response, Status};
use types::{
    aggregation::{
        aggregation_service_server::AggregationService, AggregateProofRequest,
        AggregateProofResponse, GetAggregatedDataRequest, GetAggregatedDataResponse,
        GetBatchRequest, GetBatchResponse, ProofRequest, ResponseStatus, WriteMerkleTreeRequest,
        WriteMerkleTreeResponse,
    },
    merkle_tree::MerkleTree,
};

#[tonic::async_trait]
impl AggregationService for AggregationRpc {
    async fn get_aggregated_data(
        &self,
        request: Request<GetAggregatedDataRequest>,
    ) -> Result<Response<GetAggregatedDataResponse>, Status> {
        let req = request.into_inner();
        let proof_id = req.proof_id;
        let merkle_tree_vec = db::get_merkle_tree(&self.db_pool, proof_id.clone())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        // convert merkle tree from Vec<u8> to Vec<[u8; 32]>
        let merkle_tree_leaves = merkle_tree_vec
            .chunks(32)
            .map(|chunk| chunk.try_into().unwrap())
            .collect();
        let merkle_tree = MerkleTree::new(merkle_tree_leaves);
        let proof_leaf = db::get_leaf(&self.db_pool, proof_id.clone())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let proof_status = db::get_proof_status(&self.db_pool, proof_id.clone())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let merkle_proof = merkle_tree
            .generate_proof(proof_leaf.as_slice().try_into().unwrap())
            .expect("Failed to generate proof");
        //convert merkle proof to Vec<Vec<u8>>
        let merkle_proof_vec = merkle_proof.iter().map(|leaf| leaf.to_vec()).collect();
        Ok(Response::new(GetAggregatedDataResponse {
            proof: merkle_proof_vec,
            status: proof_status,
        }))
    }

    async fn aggregate_proof(
        &self,
        request: Request<AggregateProofRequest>,
    ) -> Result<Response<AggregateProofResponse>, Status> {
        let proof_id: [u8; 32] = rand::thread_rng().gen();
        let req = request.into_inner();
        db::create_request(&self.db_pool, proof_id.to_vec(), req.proof, req.vk)
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

    async fn write_merkle_tree(
        &self,
        request: Request<WriteMerkleTreeRequest>,
    ) -> Result<Response<WriteMerkleTreeResponse>, Status> {
        let req = request.into_inner();
        let merkle_tree = req.tree;
        let batch_id = req.batch_id;
        db::write_merkle_tree(&self.db_pool, merkle_tree, batch_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(WriteMerkleTreeResponse { success: true }))
    }
}

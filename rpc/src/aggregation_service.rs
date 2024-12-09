use crate::{db, AggregationRpc};
use rand::Rng;
use tonic::{Request, Response, Status};
use types::{
    aggregation::{
        aggregation_service_server::AggregationService, AggregateProofRequest,
        AggregateProofResponse, AggregationStatusResponse, GetAggregatedDataRequest,
        GetAggregatedDataResponse, GetAggregationStatusRequest, GetAggregationStatusResponse,
        GetBatchRequest, GetBatchResponse, ProcessBatchRequest, ProcessBatchResponse,
        UpdateBatchStatusRequest, UpdateBatchStatusResponse, WriteMerkleTreeRequest,
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

        // if proof_id is not found (not in db), return empty vector and status NOT_FOUND
        let response_status = db::get_proof_status(&self.db_pool, proof_id.clone())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        if response_status == AggregationStatusResponse::NotFound {
            return Ok(Response::new(GetAggregatedDataResponse {
                proof: vec![],
                status: AggregationStatusResponse::NotFound as i32,
                tx_hash: vec![],
                chain_id: 0,
                contract_address: vec![],
            }));
        }

        let merkle_tree_vec = db::get_merkle_tree(&self.db_pool, proof_id.clone())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let merkle_tree_leaves = merkle_tree_vec
            .chunks(32)
            .map(|chunk| chunk.try_into().unwrap())
            .collect();
        let merkle_tree = MerkleTree::new(merkle_tree_leaves);

        let proof_leaf = db::get_leaf(&self.db_pool, proof_id.clone())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        // let proof_status = db::get_proof_status(&self.db_pool, proof_id.clone())
        //     .await
        //     .map_err(|e| Status::internal(e.to_string()))?;
        let merkle_proof = merkle_tree
            .generate_proof(proof_leaf.as_slice().try_into().unwrap())
            .expect("Failed to generate proof");
        let merkle_proof_vec = merkle_proof.iter().map(|leaf| leaf.to_vec()).collect();

        let (tx_hash, chain_id, contract_address) =
            db::get_tx_context(&self.db_pool, proof_id.clone())
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetAggregatedDataResponse {
            proof: merkle_proof_vec,
            status: response_status as i32,
            tx_hash,
            chain_id,
            contract_address,
        }))
    }

    async fn get_aggregation_status(
        &self,
        request: Request<GetAggregationStatusRequest>,
    ) -> Result<Response<GetAggregationStatusResponse>, Status> {
        let req = request.into_inner();
        let proof_id = req.proof_id;
        let status = db::get_proof_status(&self.db_pool, proof_id.clone())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GetAggregationStatusResponse {
            status: status as i32,
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
            req.created_after.unwrap_or(0),
            req.batch_size.unwrap_or(32),
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GetBatchResponse {
            batch_id: batch_id.to_vec(),
            proofs,
        }))
    }

    async fn process_batch(
        &self,
        request: Request<ProcessBatchRequest>,
    ) -> Result<Response<ProcessBatchResponse>, Status> {
        let req = request.into_inner();
        let batch_id = req.batch_id;
        let leaves = db::process_batch(&self.db_pool, req.proofs, batch_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ProcessBatchResponse {
            leaves: leaves.to_vec(),
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

    async fn update_batch_status(
        &self,
        request: Request<UpdateBatchStatusRequest>,
    ) -> Result<Response<UpdateBatchStatusResponse>, Status> {
        let req = request.into_inner();
        db::update_batch_status(&self.db_pool, req.batch_id, req.status)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpdateBatchStatusResponse { success: true }))
    }
}

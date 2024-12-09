-- Add migration script here
DROP TABLE IF EXISTS requests;
DROP TABLE IF EXISTS merkle_trees;
CREATE TABLE requests (
    proof_id BYTEA NOT NULL,  
    proof BYTEA NOT NULL,     
    vk BYTEA NOT NULL,      
    batch_id BYTEA NULL,       
    status BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    tx_hash BYTEA NULL,        
    chain_id BIGINT NULL,
    contract_address BYTEA NULL  
);

CREATE TABLE merkle_trees (
    batch_id BYTEA NOT NULL,    -- Changed BLOB to BYTEA
    tree BYTEA NOT NULL         -- Changed BLOB to BYTEA
);
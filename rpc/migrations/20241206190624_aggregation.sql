-- Add migration script here
CREATE TABLE requests (
    proof_id BYTEA NOT NULL,  
    proof BYTEA NOT NULL,     
    vk BYTEA NOT NULL,      
    batch_id BYTEA NULL,       
    status INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    tx_hash BYTEA NULL,        
    chain_id INTEGER NULL,
    contract_address BYTEA NULL  
);

CREATE TABLE merkle_trees (
    batch_id BYTEA NOT NULL,    
    tree BYTEA NOT NULL         
);

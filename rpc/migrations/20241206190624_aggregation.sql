-- Add migration script here
CREATE TABLE requests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proof_id BLOB NOT NULL,
    proof BLOB NOT NULL,
    vk BLOB NOT NULL,
    batch_id BLOB NULL,
    status INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE merkle_trees (
    batch_id BLOB NOT NULL,
    tree BLOB NOT NULL
);

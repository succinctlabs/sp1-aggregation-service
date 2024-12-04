// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

contract SP1AggregationVerifier {
    /// @notice The address of the SP1 verifier contract.
    /// @dev This can either be a specific SP1Verifier for a specific version, or the
    ///      SP1VerifierGateway which can be used to verify proofs for any version of SP1.
    ///      For the list of supported verifiers on each chain, see:
    ///      https://github.com/succinctlabs/sp1-contracts/tree/main/contracts/deployments
    address public verifier;

    /// @notice The root of the merkle tree of the aggregation program.
    bytes32 public merkleRoot;

    /// @notice The verification key for the aggregation program.
    bytes32 public aggregationProgramVKey;

    constructor(address _verifier, bytes32 _aggregationProgramVKey) {
        verifier = _verifier;
        aggregationProgramVKey = _aggregationProgramVKey;
    }

    /// @notice Verifies the proof for the aggregation program.
    /// @param _publicValues The public values for the aggregation program.
    /// @param _proofBytes The proof bytes for the aggregation program.
    function verifyAggregationProof(bytes calldata _publicValues, bytes calldata _proofBytes) public {
        ISP1Verifier(verifier).verifyProof(aggregationProgramVKey, _publicValues, _proofBytes);
        bytes32 newMerkleRoot = abi.decode(_publicValues, (bytes32));
        merkleRoot = newMerkleRoot;
    }

    /// @notice Verifies a Merkle proof.
    /// @param leaf The leaf node to verify.
    /// @param proof The Merkle proof to verify.
    function verifyMerkleProof(bytes32 leaf, bytes32[] calldata proof) internal view returns (bool) {
        bytes32 computedHash = leaf;
        uint256 proofLength = proof.length;

        for (uint256 i = 0; i < proofLength; i++) {
            if (computedHash < proof[i]) {
                computedHash = keccak256(abi.encodePacked(computedHash, proof[i]));
            } else {
                computedHash = keccak256(abi.encodePacked(proof[i], computedHash));
            }
        }

        return computedHash == merkleRoot;
    }

    /// @notice Verifies the inclusion of the user's proof in the aggregation proof.
    /// @param _programVKey The verification key for the user's program.
    /// @param _publicValues The public values for the user's program.
    /// @param _merkleProof The Merkle proof for the user's proof against the aggregation Merkle tree.
    function verifyProof(bytes calldata _programVKey, bytes calldata _publicValues, bytes32[] calldata _merkleProof) public view {
        // Hash pair (programVk, publicValues) Sha256 digest
        bytes32 pairHash = sha256(abi.encodePacked(_programVKey, _publicValues));

        // Verify the Merkle proof
        bool isProofValid = verifyMerkleProof(pairHash, _merkleProof);
        require(isProofValid, "Invalid Merkle proof");
    }
}


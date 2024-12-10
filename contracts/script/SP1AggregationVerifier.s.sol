// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {BaseScript} from "../lib/sp1-contracts/contracts/script/utils/Base.s.sol";
import {SP1AggregationVerifier} from "../src/SP1AggregationVerifier.sol";

contract SP1AggregationVerifierScript is BaseScript {
    string internal constant KEY = "SP1_AGGREGATION_VERIFIER";

    function run() external multichain(KEY) broadcaster {
        // Read deployment inputs
        address verifier = readAddress("VERIFIER");
        bytes32 aggregationProgramVKey = readBytes32("AGGREGATION_PROGRAM_VKEY");

        // Deploy the SP1AggregationVerifier contract
        address verifierAddress = address(
            new SP1AggregationVerifier(verifier, aggregationProgramVKey)
        );

        // Save the deployed address
        writeAddress(KEY, verifierAddress);
    }
}

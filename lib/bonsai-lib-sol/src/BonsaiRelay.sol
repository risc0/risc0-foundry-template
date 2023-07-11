// Copyright 2023 Risc0, Inc.
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import {IBonsaiRelay} from "./IBonsaiRelay.sol";

/// @notice A mock Bonsai relay for local testing
contract BonsaiRelay is IBonsaiRelay {
    // Flag to signal if the contract has been initialized.
    // Useful when putting this contract behind a Proxy.
    bool initialized;
    // Initial config
    address public owner;
    // Events
    event CallbackRequest(
        address account,
        bytes32 image_id,
        bytes input,
        address callback_contract,
        bytes4 function_selector,
        uint64 gas_limit
    );
    // Callback
    struct Callback {
        address callback_contract;
        SnarkProof proof;
        bytes payload;
        uint64 gas_limit;
    }
    // Snark Proof
    struct SnarkProof {
        uint[2] a;
        uint[2][2] b;
        uint[2] c;
        uint[4] pubSignals;
    }
    // Initiate Contract
    constructor() {
        initialize();
    }
    // Initalize the owner and the genesis_range_root
    function initialize() public {
        require(initialized == false, "contract already initialized");
        owner = address(msg.sender);
        initialized = true;
    }
    // Submit request
    function requestCallback(
        bytes32 image_id,
        bytes calldata input,
        address callback_contract,
        bytes4 function_selector,
        uint64 gas_limit
    ) external {
        // Emit event
        emit CallbackRequest(msg.sender, image_id, input, callback_contract, function_selector, gas_limit);
    }
    // Submit proofs
    function invoke_callback(Callback[] calldata callbacks) external returns (bool[] memory invocation_results) {
        require(msg.sender == owner, "Denied");
        invocation_results = new bool[](callbacks.length);
        for (uint i = 0; i < callbacks.length; i++) {
            // uint payload_size = callbacks[i].payload.length;
            // // Validate Bonsai proof
            // bytes32 image_id = bytes32(callbacks[i].payload[payload_size-32:]);
            // bytes32 journal_hash = sha256(callbacks[i].payload[4:payload_size-32]);
            // require(bpn_contract.verify(image_id, journal_hash, callbacks[i].journal_inclusion_proof), "Invalid Bonsai Proof");
            
            // invoke callback
            (invocation_results[i], ) =callbacks[i].callback_contract.call{gas: callbacks[i].gas_limit}(callbacks[i].payload);
        }
    }
}
// Copyright 2023 Risc0, Inc.
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

contract BonsaiRelayContract {
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
        bytes32[] journal_inclusion_proof;
        bytes payload;
        uint64 gas_limit;
    }
    // Initiate Contract
    constructor() {
        owner = address(msg.sender);
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
    function invoke_callbacks(Callback[] calldata callbacks) external returns (bool[] memory invocation_results) {
        require(msg.sender == owner, "Denied");
        invocation_results = new bool[](callbacks.length);
        for (uint i = 0; i < callbacks.length; i++) {
            // invoke callback
            (invocation_results[i], ) = callbacks[i].callback_contract.call{gas: callbacks[i].gas_limit}(callbacks[i].payload);
        }
    }
}

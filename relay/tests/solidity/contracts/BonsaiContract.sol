// Copyright 2023 Risc0, Inc.
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

contract BonsaiContract {
    // the height of the latest block processed so far.
    uint256 public latest_block_height;

    constructor(uint256 block_height) {
        latest_block_height = block_height;
    }

    function set_latest_block(uint256 block_height) external {
        latest_block_height = block_height;
    }
}

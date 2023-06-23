// Copyright 2023 Risc0, Inc.
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

interface IBPN {
    function verify(bytes32 image_id, bytes32 journal_hash, bytes32[] calldata proof) external view returns (bool);
}

// SPDX-License-Identifier: MIT

pragma solidity ^0.8.20;

import {console2} from "forge-std/console2.sol";
import {Test} from "forge-std/Test.sol";
import {MerkleTreeWithHistory} from "../contracts/MerkleTreeWithHistory.sol";

contract MerkleTreeTest is Test, MerkleTreeWithHistory(10) {
    function setUp() public {}

    function test_insertion() public {
        _insert(bytes32(0x0));
        _insert(
            bytes32(
                0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            )
        );
        // test against values caluculated locally using the Rust incremental merkle tree impl
        assert(
            roots[0] ==
                0x506d86582d252405b840018792cad2bf1259f1ef5aa5f887e13cb2f0094f51e1
        );
        assert(
            roots[1] ==
                0xffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b
        );
        assert(
            roots[2] ==
                0x606f20333f7003fc7839a11ddbf8a3d85d3f35c5b993889dde79aa2caf13d61d
        );
    }
}

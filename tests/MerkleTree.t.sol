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
        // assert(
        //     roots[0] ==
        //         0xffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b
        // );
        console2.logBytes32(roots[0]);
        console2.logBytes32(roots[1]);
        console2.logBytes32(roots[2]);
        console2.logBytes32(roots[3]);
        // assert(
        //     roots[1] ==
        //         0xffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b
        // );
    }
}

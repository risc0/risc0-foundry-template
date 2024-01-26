// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {BonsaiTest} from "bonsai/BonsaiTest.sol";
import {RiscZeroGroth16VerifierTest} from "./RiscZeroGroth16VerifierTest.sol";
import "../contracts/BonsaiStarter.sol";

contract EvenNumberTest is BonsaiTest {
    EvenNumber public evenNumber;

    function setUp() public {
        bytes32 imageId = queryImageId("IS_EVEN");
        RiscZeroGroth16VerifierTest verifierTest = new RiscZeroGroth16VerifierTest();
        evenNumber = new EvenNumber(verifierTest, imageId);
        assertEq(evenNumber.get(), 0);
    }

    function testSet() public {
        uint256 number = 12345678;
        bytes32 imageId = queryImageId("IS_EVEN");
        (
            bytes memory journal,
            bytes32 post_state_digest,
            bytes memory seal
        ) = queryImageOutputAndSeal(imageId, abi.encode(number));
        assertEq(abi.decode(journal, (uint256)), number);
        evenNumber.set(abi.decode(journal, (uint256)), post_state_digest, seal);
        assertEq(evenNumber.get(), number);
    }
}

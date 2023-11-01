// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.21;

import {Starter, ProofOutput} from "contracts/Starter.sol";
import {MockVerifier} from "contracts/verify/MockVerifier.sol";
import {ControlID, RiscZeroG16Verifier} from "contracts/verify/groth16/RiscZeroG16Verifier.sol";
import {IRiscZeroVerifier} from "contracts/verify/IRiscZeroVerifier.sol";
import {Proof} from "contracts/verify/IRiscZeroVerifier.sol";
import {BonsaiTest} from "contracts/test/BonsaiCheats.sol";
import {Test} from "forge-std/Test.sol";

contract StarterTest is Test, BonsaiTest {
    bytes32 internal IMAGE_ID;
    IRiscZeroVerifier internal verifier;
    Starter internal starter;

    function setUp() public {
        if (vm.envOr("RISC0_DEV_MODE", true)) {
            IMAGE_ID = bytes32("1");
            verifier = new MockVerifier(ControlID.CONTROL_ID_0, ControlID.CONTROL_ID_1);
        } else {
            IMAGE_ID = queryImageId("FIBONACCI");
            verifier = new RiscZeroG16Verifier(ControlID.CONTROL_ID_0, ControlID.CONTROL_ID_1);
        }

        starter = new Starter(address(verifier), IMAGE_ID);
    }

    function test_Mock() public {
        uint256 record = 610;
        ProofOutput memory output = ProofOutput(15, record);
        Proof memory proof = Proof(bytes("123"), bytes32("123"), abi.encode(output));
        starter.submit(proof);
        assertEq(starter.record(), record);
    }
}

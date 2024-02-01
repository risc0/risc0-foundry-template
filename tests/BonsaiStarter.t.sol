// Copyright 2024 RISC Zero, Inc.
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

pragma solidity ^0.8.20;

import {BonsaiTest} from "bonsai/BonsaiTest.sol";
import {console2} from "forge-std/console2.sol";
import {IRiscZeroVerifier} from "bonsai/IRiscZeroVerifier.sol";
import {ControlID, RiscZeroGroth16Verifier} from "bonsai/groth16/RiscZeroGroth16Verifier.sol";
import {RiscZeroGroth16VerifierTest} from "./RiscZeroGroth16VerifierTest.sol";
import {EvenNumber} from "../contracts/EvenNumber.sol";

contract EvenNumberTest is BonsaiTest {
    EvenNumber public evenNumber;
    bytes32 private imageId;

    function setUp() public {
        imageId = queryImageId("IS_EVEN");
        IRiscZeroVerifier verifier = deployRiscZeroGroth16Verifier();
        evenNumber = new EvenNumber(verifier, imageId);
        assertEq(evenNumber.get(), 0);
    }

    function testSet() public {
        uint256 number = 12345678;
        (bytes memory journal, bytes32 post_state_digest, bytes memory seal) =
            queryImageOutputAndSeal(imageId, abi.encode(number));
        evenNumber.set(abi.decode(journal, (uint256)), post_state_digest, seal);
        assertEq(evenNumber.get(), number);
    }

    /// @notice Deploy either a test or fully verifying `RiscZeroGroth16Verifier` depending on RISC0_DEV_MODE.
    function deployRiscZeroGroth16Verifier() internal returns (IRiscZeroVerifier) {
        if (vm.envOr("RISC0_DEV_MODE", false) == false) {
            IRiscZeroVerifier verifier = new RiscZeroGroth16Verifier(ControlID.CONTROL_ID_0, ControlID.CONTROL_ID_1);
            console2.log("Deployed RiscZeroGroth16Verifier to", address(verifier));
            return verifier;
        } else {
            IRiscZeroVerifier verifier = new RiscZeroGroth16VerifierTest();
            console2.log("Deployed RiscZeroGroth16VerifierTest to", address(verifier));
            return verifier;
        }
    }
}

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

pragma solidity ^0.8.19;

import {BonsaiCheats} from "bonsai/BonsaiCheats.sol";
import {Script} from "forge-std/Script.sol";
import {console2} from "forge-std/console2.sol";
import {IRiscZeroVerifier} from "bonsai/IRiscZeroVerifier.sol";
import {ControlID, RiscZeroGroth16Verifier} from "bonsai/groth16/RiscZeroGroth16Verifier.sol";

import "../contracts/BonsaiStarter.sol";

contract EvenNumberDeploy is Script, BonsaiCheats {
    function run() external {
        uint256 deployerKey = vm.envOr("ETH_WALLET_PRIVATE_KEY", uint256(0));

        vm.startBroadcast(deployerKey);

        IRiscZeroVerifier verifier = new RiscZeroGroth16Verifier(ControlID.CONTROL_ID_0, ControlID.CONTROL_ID_1);
        console2.log("Deployed RiscZeroGroth16Verifier to", address(verifier));

        bytes32 imageId = queryImageId("IS_EVEN");
        EvenNumber evenNumber = new EvenNumber(verifier, imageId);
        console2.log("Deployed EvenNumber to", address(evenNumber));

        vm.stopBroadcast();
    }
}

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

import {Script} from "forge-std/Script.sol";
import "forge-std/Test.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {RiscZeroGroth16Verifier} from "risc0/groth16/RiscZeroGroth16Verifier.sol";
import {ControlID} from "risc0/groth16/ControlID.sol";

import {EvenNumber} from "../contracts/EvenNumber.sol";

/// @notice Deployment script for the RISC Zero starter project.
/// @dev Use the following environment variable to control the deployment:
///     * ETH_WALLET_PRIVATE_KEY private key of the wallet to be used for deployment.
///
/// See the Foundry documentation for more information about Solidity scripts.
/// https://book.getfoundry.sh/tutorials/solidity-scripting
contract EvenNumberDeploy is Script {
    using stdToml for string;

    string constant DEFAULT_PROFILE = "DEFAULT_PROFILE";
    IRiscZeroVerifier verifier;

    function run() external {
        // read and log the chainID
        uint256 chainId = block.chainid;
        console2.log("You are deploying on ChainID %d", chainId);

        uint256 deployerKey = uint256(vm.envBytes32("ETH_WALLET_PRIVATE_KEY"));

        vm.startBroadcast(deployerKey);

        string memory configProfile = vm.envOr("CONFIG_PROFILE", DEFAULT_PROFILE);
        if (keccak256(abi.encodePacked(configProfile)) != keccak256(abi.encodePacked(DEFAULT_PROFILE))) {
            string memory configData = vm.readFile("script/config.toml");
            string memory profile = string.concat(".profile.", configProfile);
            console2.log("Deploying using config profile:", configProfile);
            address verifierAddress = configData.readAddress(string.concat(profile, ".verifierAddress"));
            verifier = IRiscZeroVerifier(verifierAddress);
            console2.log("Using IRiscZeroVerifier contract deployed at", verifierAddress);
        } else {
            verifier = new RiscZeroGroth16Verifier(ControlID.CONTROL_ROOT, ControlID.BN254_CONTROL_ID);
            console2.log("Deployed IRiscZeroVerifier to", address(verifier));
        }

        EvenNumber evenNumber = new EvenNumber(verifier);
        console2.log("Deployed EvenNumber to", address(evenNumber));

        vm.stopBroadcast();
    }
}

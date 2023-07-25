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

pragma solidity ^0.8.17;

import {Script} from "forge-std/Script.sol";
import {console2} from "forge-std/console2.sol";
import {IBonsaiRelay} from "bonsai/IBonsaiRelay.sol";
import {BonsaiRelay} from "bonsai/BonsaiRelay.sol";
import {BonsaiCheats} from "bonsai/BonsaiCheats.sol";
import {BonsaiTestRelay} from "bonsai/BonsaiTestRelay.sol";
import {RiscZeroGroth16Verifier} from "bonsai/groth16/RiscZeroGroth16Verifier.sol";
import {IRiscZeroVerifier} from "bonsai/IRiscZeroVerifier.sol";
import {IVotes} from "openzeppelin/contracts/governance/utils/IVotes.sol";

contract BonsaiDeploy is Script, BonsaiCheats {
    /// @notice use vm.startBroadcast to begin recording deploy transactions.
    function startBroadcast() internal {
        address deployerAddr = vm.envOr("DEPLOYER_ADDRESS", address(0));
        uint256 deployerKey = vm.envOr("DEPLOYER_PRIVATE_KEY", uint256(0));

        if (deployerAddr != address(0) && deployerKey != uint256(0)) {
            revert("only one of DEPLOYER_ADDRESS or DEPLOYER_PRIVATE_KEY should be set");
        }
        if (deployerAddr != address(0)) {
            vm.startBroadcast(deployerAddr);
        } else if (deployerKey != uint256(0)) {
            vm.startBroadcast(deployerKey);
        } else if (block.chainid == 31337) {
            // On an Anvil local testnet, use the first private key by default.
            deployerKey = uint256(0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80);
            vm.startBroadcast(deployerKey);
        } else {
            revert("specify a deployer with either DEPLOYER_ADDRESS or DEPLOYER_PRIVATE_KEY");
        }
    }

    /// @notice Deploy a fully verifying BonsaiRelay contract instance if an address is not already specified.
    /// @dev Relay is stateless and owner-less.
    function deployBonsaiVerifyingRelay() internal returns (IBonsaiRelay) {
        IBonsaiRelay bonsaiRelay;
        address relayAddr = vm.envOr("DEPLOY_RELAY_ADDRESS", address(0));
        if (relayAddr != address(0)) {
            console2.log("Using IBonsaiRelay at ", address(relayAddr));
            bonsaiRelay = IBonsaiRelay(relayAddr);
        } else {
            // Deploy an IRiscZeroVerifier contract instance. Relay is stateless and owner-less.
            IRiscZeroVerifier verifier;
            address verifierAddr = vm.envOr("DEPLOY_VERFIER_ADDRESS", address(0));
            if (verifierAddr != address(0)) {
                console2.log("Using IRiscZeroVerifier at ", address(verifierAddr));
                verifier = IRiscZeroVerifier(verifierAddr);
            } else {
                verifier = new RiscZeroGroth16Verifier();
                console2.log("Deployed RiscZeroGroth16Verifier to ", address(verifier));
            }

            bonsaiRelay = new BonsaiRelay(verifier);
            console2.log("Deployed BonsaiRelay to ", address(bonsaiRelay));
        }
        return bonsaiRelay;
    }

    /// @notice Deploy a BonsaiTestRelay contract instance if an address is not already specified.
    /// @dev Relay is stateless and owner-less.
    function deployBonsaiTestRelay() internal returns (IBonsaiRelay) {
        IBonsaiRelay bonsaiRelay;
        address relayAddr = vm.envOr("DEPLOY_RELAY_ADDRESS", address(0));
        if (relayAddr != address(0)) {
            console2.log("Using BonsaiRelay at ", address(relayAddr));
            bonsaiRelay = IBonsaiRelay(relayAddr);
        } else {
            // BonsaiTestRelay SHOULD ONLY BE DEPLOYED IN TEST SCENARIOS.
            // Use a long and unweildy environment variable name for overriding
            // the expected chain ID for the test relay so that it is hard to
            // trigger without thinking about it.
            bonsaiRelay = new BonsaiTestRelay(vm.envOr("DEPLOY_BONSAI_TEST_RELAY_EXPECTED_CHAIN_ID", uint256(31337)));
            console2.log("Deployed BonsaiTestRelay to ", address(bonsaiRelay));
        }
        return bonsaiRelay;
    }

    /// @notice Deploy either a test or fully verifying relay depending on BONSAI_PROVING.
    /// @dev Relay is stateless and owner-less.
    function deployBonsaiRelay() internal returns (IBonsaiRelay) {
        if (proverMode() == ProverMode.Bonsai) {
            return deployBonsaiVerifyingRelay();
        } else if (proverMode() == ProverMode.Local) {
            return deployBonsaiTestRelay();
        }
        revert("invalid value for proverMode");
    }

    /// @notice If DEPLOY_UPLOAD_IMAGES is true, upload all guests defined in the methods directory to Bonsai.
    /// @dev If DEPLOY_UPLOAD_IMAGES is not set, defaults to true.
    function uploadImages() internal {
        if (vm.envOr("DEPLOY_UPLOAD_IMAGES", false)) {
            bytes32[] memory imageIds = uploadAllImages();
            if (imageIds.length == 0) {
                console2.log("No images uploaded to Bonsai");
            }
            for (uint256 i = 0; i < imageIds.length; i++) {
                console2.log("Uploaded guest image to Bonsai", vm.toString(imageIds[i]));
            }
        }
    }
}

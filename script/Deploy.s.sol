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
import {BonsaiCheats} from "bonsai/BonsaiCheats.sol";

import {BonsaiDeploy} from "./BonsaiDeploy.sol";
import {BonsaiStarter} from "../contracts/BonsaiStarter.sol";

/// @notice deployment script for the Bonsai Governor and it's dependencies.
/// @dev Use the following environment variables to control the deployment:
///     * DEPLOYER_ADDRESS address of the wallet to be used for sending deploy transactions.
///         Must be unlocked on the RPC provider node.
///     * DEPLOYER_PRIVATE_KEY private key of the wallet to be used for deployment.
///         Alternative to using DEPLOYER_ADDRESS.
///     * DEPLOY_VERFIER_ADDRESS address of a predeployed IRiscZeroVerifier contract.
///         If not specified and also DEPLOY_RELAY_ADDRESS is not specified,
///         a new RiscZeroGroth16Verifier will be deployed.
///     * DEPLOY_RELAY_ADDRESS address of a predeployed BonsaiRelay contract.
///         If not specified, a new BonsaiRelay will be deployed.
///     * DEPLOY_UPLOAD_IMAGES true or false indicating whether to upload the zkVM guest images to
///         Bonsai. Default is true if BONSAI_API_URL is set.
///     * BONSAI_PROVING indicates what mode of proving is being used and decides what relay
///         contract to deploy.
///         * If BONSAI_PROVING = local: The mock BonsaiTestRelay contract will be used.
///         * If BONSAI_PROVING = bonsai: The fully verifying BonsaiRelay contract will be used.
contract Deploy is Script, BonsaiCheats, BonsaiDeploy {
    /// @notice Deploy application, with a pointer to the Bonsai relay contract and image ID.
    function deployApp(IBonsaiRelay bonsaiRelay) internal override {
        bytes32 imageId = queryImageId("FIBONACCI");
        console2.log("Image ID for FIBONACCI is ", vm.toString(imageId));
        BonsaiStarter app = new BonsaiStarter(bonsaiRelay, imageId);
        console2.log("Deployed BonsaiStarter to ", address(app));
    }
}

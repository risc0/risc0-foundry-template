// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.17;

import {Script} from "forge-std/Script.sol";
import {console2} from "forge-std/console2.sol";
import {IBonsaiRelay} from "bonsai/IBonsaiRelay.sol";
import {BonsaiCheats} from "bonsai/BonsaiCheats.sol";

import {BonsaiDeploy} from "./BonsaiDeploy.sol";
import {BonsaiStarter} from "../contracts/BonsaiStarter.sol";

/// @notice Deployment script for the BonsaiStarter project.
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
///         Bonsai. Default is false.
///     * RISC0_DEV_MODE indicates what mode of proving is being used and decides which relay
///         contract to deploy.
///         * If "true": The mock BonsaiTestRelay contract will be used.
///         * If "false" or unset: The fully verifying BonsaiRelay contract will be used.
contract Deploy is Script, BonsaiCheats, BonsaiDeploy {
    function run() external {
        startBroadcast();
        IBonsaiRelay bonsaiRelay = deployBonsaiRelay();
        uploadImages();

        // TEMPLATE: Modify this block to match your expected deployment.
        bytes32 imageId = queryImageId("FIBONACCI");
        console2.log("Image ID for FIBONACCI is ", vm.toString(imageId));
        BonsaiStarter app = new BonsaiStarter(bonsaiRelay, imageId);
        console2.log("Deployed BonsaiStarter to ", address(app));

        vm.stopBroadcast();
    }
}

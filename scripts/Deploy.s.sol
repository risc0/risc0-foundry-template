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

import "../lib/forge-std/src/Script.sol";
import "../relay/contracts/BonsaiRelay.sol";
import "../contracts/BonsaiStarter.sol";
import "../lib/bonsai-lib-sol/src/BonsaiCheats.sol";

contract Relay is Script, BonsaiCheats {
    function run() external {
        uint256 relayPrivateKey =
            vm.envOr("RELAY_PRIVATE_KEY", uint256(0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80));
        vm.startBroadcast(relayPrivateKey);

        // Deploy a Relay contract instance
        BonsaiRelay relayContract = new BonsaiRelay();
        
        IBonsaiRelay bonsaiRelay = IBonsaiRelay(address(relayContract));
        console.logAddress(address(bonsaiRelay));

        vm.stopBroadcast();
    }
}

contract Starter is Script, BonsaiCheats {
    function run() external {
        address relayContract =
            vm.envAddress("RELAY_ADDRESS");
        string memory bonsaiApiUrl =
            vm.envString("BONSAI_API_URL");
        string memory bonsaiApiKey =
            vm.envString("BONSAI_API_KEY");
        string memory methodName =
            vm.envString("METHOD_NAME");
        uint256 relayPrivateKey =
            vm.envOr("RELAY_PRIVATE_KEY", uint256(0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80));
        vm.startBroadcast(relayPrivateKey);

        IBonsaiRelay bonsaiRelay = IBonsaiRelay(relayContract);
        bytes32 imageId = uploadImage(methodName, bonsaiApiUrl, bonsaiApiKey);
        
        // Deploy a new starter instance (or replace with deployment of your own contract here)
        BonsaiStarter starter = new BonsaiStarter(bonsaiRelay, imageId);

        console.logBytes32(imageId);
        console.logAddress(address(starter));

        vm.stopBroadcast();
    }
}
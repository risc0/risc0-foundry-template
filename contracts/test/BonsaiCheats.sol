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

import {StdCheatsSafe} from "forge-std/StdCheats.sol";
import {CommonBase} from "forge-std/Base.sol";

abstract contract BonsaiTest is CommonBase, StdCheatsSafe {
    /// @notice Returns the image id of the guest with the specified name.
    function queryImageId(string memory binaryName) internal returns (bytes32) {
        string[] memory imageRunnerInput = new string[](5);
        uint256 i = 0;
        imageRunnerInput[i++] = "cargo";
        imageRunnerInput[i++] = "run";
        imageRunnerInput[i++] = "-q";
        imageRunnerInput[i++] = "query";
        imageRunnerInput[i++] = binaryName;
        return abi.decode(vm.ffi(imageRunnerInput), (bytes32));
    }
}

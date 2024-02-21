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

import {IRiscZeroVerifier} from "bonsai/IRiscZeroVerifier.sol";
import {ImageID} from "./ImageID.sol"; // auto-generated contract after running `cargo build`.

/// @title A starter application using Bonsai.
/// @notice This basic application holds a number, guaranteed to be even.
/// @dev This contract demonstrates one pattern for offloading the computation of an expensive
///      or difficult to implement function to a RISC Zero guest running on Bonsai.
contract EvenNumber {
    /// @notice RISC Zero verifier contract address.
    IRiscZeroVerifier immutable verifier;
    /// @notice Image ID of the only zkVM binary to accept verification from.
    bytes32 immutable imageId;
    /// @notice A number that is guaranteed, by the RISC Zero zkVM, to be even.
    ///         It can be set by calling the `set` function.
    uint256 number;

    /// @notice Initialize the contract, binding it to a specified RISC Zero verifier and guest image.
    constructor(IRiscZeroVerifier _verifier) {
        verifier = _verifier;
        imageId = ImageID.IS_EVEN_ID;
        number = 0;
    }

    /// @notice Set the even number stored on the contract. Requires a RISC Zero proof that the number is even.
    function set(uint256 x, bytes32 postStateDigest, bytes calldata seal) public {
        require(verifier.verify(seal, imageId, postStateDigest, sha256(abi.encode(x))));
        number = x;
    }

    /// @notice Returns the number stored.
    function get() public view returns (uint256) {
        return number;
    }
}
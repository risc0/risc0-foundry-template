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

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {IRiscZeroVerifier} from "bonsai/IRiscZeroVerifier.sol";

contract EvenNumber {
    IRiscZeroVerifier verifier;
    bytes32 imageId;
    uint256 number;

    constructor(IRiscZeroVerifier _verifier, bytes32 _imageId) {
        verifier = _verifier;
        imageId = _imageId;
        number = 0;
    }

    function set(uint256 x, bytes32 postStateDigest, bytes calldata seal) public {
        require(verifier.verify(seal, imageId, postStateDigest, sha256(abi.encode(x))));
        number = x;
    }

    function get() public view returns (uint256) {
        return number;
    }
}

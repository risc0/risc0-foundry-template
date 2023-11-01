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

import {
    IRiscZeroVerifier,
    Receipt,
    ReceiptMetadata,
    ReceiptMetadataLib,
    ExitCode,
    SystemExitCode
} from "./IRiscZeroVerifier.sol";

contract MockVerifier is IRiscZeroVerifier {
    using ReceiptMetadataLib for ReceiptMetadata;

    uint256 internal control_id_0;
    uint256 internal control_id_1;

    constructor(uint256 _control_id_0, uint256 _control_id_1) {
        control_id_0 = _control_id_0;
        control_id_1 = _control_id_1;
    }

    function verify(Receipt memory) public pure returns (bool) {
        return true;
    }

    function verify(bytes memory seal, bytes32 imageId, bytes32 postStateDigest, bytes32 journalHash)
        public
        pure
        returns (bool)
    {
        Receipt memory receipt = Receipt(
            seal, ReceiptMetadata(imageId, postStateDigest, ExitCode(SystemExitCode.Halted, 0), bytes32(0), journalHash)
        );
        return verify(receipt);
    }

    function verify(bytes memory seal, bytes32 imageId, bytes32 postStateDigest, bytes calldata journal)
        public
        pure
        returns (bool)
    {
        return verify(seal, imageId, postStateDigest, sha256(journal));
    }
}

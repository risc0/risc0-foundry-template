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

import {Receipt, IRiscZeroVerifier, Proof} from "./verify/IRiscZeroVerifier.sol";

struct ProofOutput {
    uint256 n;
    uint256 result;
}

contract Starter {
    uint256 public record;
    address public winner;

    bytes32 immutable imageId;
    IRiscZeroVerifier immutable verifier;

    event NewWinner(address indexed winner, uint256 indexed record);

    constructor(address _verifier, bytes32 _imageId) {
        record = 0;
        winner = address(0);
        verifier = IRiscZeroVerifier(_verifier);
        imageId = _imageId;
    }

    function submit(Proof calldata proof) public {
        require(verifier.verify(proof.seal, imageId, proof.postStateDigest, proof.journal), "Invalid receipt/proof");
        ProofOutput memory output = abi.decode(proof.journal, (ProofOutput));
        require(output.result > record, "Proof output not larger than current record");
        record = output.result;
        winner = msg.sender;
        emit NewWinner(winner, record);
    }
}

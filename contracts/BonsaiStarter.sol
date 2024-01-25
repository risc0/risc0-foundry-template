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

    function set(uint256 x, bytes calldata seal, bytes32 postStateDigest) public {
        require(verifier.verify(seal, imageId, postStateDigest, sha256(abi.encode(x))));
        number = x;
    }

    function get() public view returns (uint256) {
        return number;
    }
}

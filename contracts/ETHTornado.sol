// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import "./Tornado.sol";

contract ETHTornado is Tornado {
    constructor(
        IRiscZeroVerifier _verifier,
        uint256 _denomination,
        uint32 _merkleTreeHeight
    ) Tornado(_verifier, _denomination, _merkleTreeHeight) {}

    function _processDeposit() internal override {
        require(
            msg.value == denomination,
            "Please send `mixDenomination` ETH along with transaction"
        );
    }

    function _processWithdraw(address payable _recipient) internal override {
        // sanity checks
        require(
            msg.value == 0,
            "Message value is supposed to be zero for ETH instance"
        );

        (bool success, ) = _recipient.call{value: denomination}("");
        require(success, "payment to _recipient did not go thru");
    }
}

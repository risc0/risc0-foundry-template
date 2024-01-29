// Copyright 2024 RISC Zero, Inc.
//
// The RiscZeroGroth16Verifier is a free software: you can redistribute it
// and/or modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
//
// The RiscZeroGroth16Verifier is distributed in the hope that it will be
// useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
// Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// the RiscZeroGroth16Verifier. If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0

pragma solidity ^0.8.9;

import {SafeCast} from "openzeppelin/contracts/utils/math/SafeCast.sol";

import {Groth16Verifier} from "bonsai/groth16/Groth16Verifier.sol";
import {
    ExitCode,
    IRiscZeroVerifier,
    Output,
    OutputLib,
    Receipt,
    ReceiptClaim,
    ReceiptClaimLib,
    SystemExitCode
} from "bonsai/IRiscZeroVerifier.sol";

/// @notice Groth16 verifier contract for RISC Zero receipts of execution.
contract RiscZeroGroth16VerifierTest is IRiscZeroVerifier, Groth16Verifier {
    uint256 public immutable CONTROL_ID_0;
    uint256 public immutable CONTROL_ID_1;

    constructor() {
        CONTROL_ID_0 = 0;
        CONTROL_ID_1 = 0;
    }

    /// @inheritdoc IRiscZeroVerifier
    function verify(bytes calldata seal, bytes32, /*imageId*/ bytes32 postStateDigest, bytes32 /*journalDigest*/ )
        public
        view
        returns (bool)
    {
        // Require that the seal be specifically empty.
        // Reject if the caller may have sent a real seal.
        return CONTROL_ID_0 == 0 && CONTROL_ID_1 == 0 && seal.length == 0 && postStateDigest == bytes32(0);
    }

    /// @inheritdoc IRiscZeroVerifier
    function verify_integrity(Receipt memory /*receipt*/ ) public view returns (bool) {
        return CONTROL_ID_0 == 0 && CONTROL_ID_1 == 0;
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {BonsaiTest} from "bonsai/BonsaiTest.sol";
import {console2} from "forge-std/console2.sol";
import {IRiscZeroVerifier} from "bonsai/IRiscZeroVerifier.sol";
import {ControlID, RiscZeroGroth16Verifier} from "bonsai/groth16/RiscZeroGroth16Verifier.sol";
import {RiscZeroGroth16VerifierTest} from "./RiscZeroGroth16VerifierTest.sol";
import "../contracts/BonsaiStarter.sol";

contract EvenNumberTest is BonsaiTest {
    EvenNumber public evenNumber;

    function setUp() public {
        bytes32 imageId = queryImageId("IS_EVEN");
        IRiscZeroVerifier verifier = deployRiscZeroGroth16Verifier();
        evenNumber = new EvenNumber(verifier, imageId);
        assertEq(evenNumber.get(), 0);
    }

    function testSet() public {
        uint256 number = 12345678;
        bytes32 imageId = queryImageId("IS_EVEN");
        (
            bytes memory journal,
            bytes32 post_state_digest,
            bytes memory seal
        ) = queryImageOutputAndSeal(imageId, abi.encode(number));
        evenNumber.set(abi.decode(journal, (uint256)), post_state_digest, seal);
        assertEq(evenNumber.get(), number);
    }

    /// @notice Deploy either a test or fully verifying `RiscZeroGroth16Verifier` depending on RISC0_DEV_MODE.
    function deployRiscZeroGroth16Verifier()
        internal
        returns (IRiscZeroVerifier)
    {
        if (vm.envOr("RISC0_DEV_MODE", false) == false) {
            IRiscZeroVerifier verifier = new RiscZeroGroth16Verifier(
                ControlID.CONTROL_ID_0,
                ControlID.CONTROL_ID_1
            );
            console2.log(
                "Deployed RiscZeroGroth16Verifier to",
                address(verifier)
            );
            return verifier;
        } else {
            IRiscZeroVerifier verifier = new RiscZeroGroth16VerifierTest();
            console2.log(
                "Deployed RiscZeroGroth16VerifierTest to",
                address(verifier)
            );
            return verifier;
        }
    }
}

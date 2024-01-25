// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Script} from "forge-std/Script.sol";
import {console2} from "forge-std/console2.sol";
import {IRiscZeroVerifier} from "bonsai/IRiscZeroVerifier.sol";
import {ControlID, RiscZeroGroth16Verifier} from "bonsai/groth16/RiscZeroGroth16Verifier.sol";

import "../contracts/BonsaiStarter.sol";

contract EvenNumberDeploy is Script {
    function run() external {
        uint256 deployerKey = vm.envOr("ETH_WALLET_PRIVATE_KEY", uint256(0));

        vm.startBroadcast(deployerKey);

        IRiscZeroVerifier verifier = new RiscZeroGroth16Verifier(ControlID.CONTROL_ID_0, ControlID.CONTROL_ID_1);
        console2.log("Deployed RiscZeroGroth16Verifier to", address(verifier));

        EvenNumber evenNumber =
            new EvenNumber(verifier, 0xa233b08506289266e2209d24fee095c44564e97eb303547c25220a7a0cd96757);
        console2.log("Deployed EvenNumber to", address(evenNumber));

        vm.stopBroadcast();
    }
}

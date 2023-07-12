// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {console2} from "forge-std/console2.sol";

import {
    RiscZeroVerifier,
    Seal,
    Receipt as RiscZeroReceipt,
    ReceiptMetadata,
    ReceiptMetadataLib,
    ExitCode,
    SystemExitCode
} from "../contracts/RiscZeroVerifier.sol";

contract RiscZeroVerifierTest is Test {
    using ReceiptMetadataLib for ReceiptMetadata;

    // A known-good SNARK proof generated for the BonsaiGovernor contract by Bonsai.
    RiscZeroReceipt internal TEST_RECEIPT = RiscZeroReceipt(
        Seal(
            [
                0x2c66ed69d8487dcfb7fb5a6471c526a73839f21eadd07bdf4181f6209b3e8026,
                0x08664749c2d46278b94cc8662a529ebdc059a7244451cb87581e0be5d55b86f9
            ],
            [
                [
                    0x2a2e27079fa0bf2ed33e7cf66c34f016bfee00d70a5766212e094730dd6a0304,
                    0x2d6bfd9239208d6b6d8dca568802930f031bee803e70da101c431ced1a9a0070
                ],
                [
                    0x2d7c01523d70f7b7309e71355ace8ad6355122b6abeece9f23458160df95b8e5,
                    0x1d11d24867746103b3fba77e09db47a93f452753b0a59ff42beb8dca7d4f751b
                ]
            ],
            [
                0x2c56e66032a383422e57c2ed37786a15eb0f647cf7574816f299e5399dec0c89,
                0x271eba28989da164e5c6894002004b69ec1d841e7c91d2c3929a7b4b00944308
            ]
        ),
        ReceiptMetadata(
            bytes32(0xfb745ba9d74cce252c57ef0991a2e7cfc46304a1ece848cd98e5e57400f0707c),
            bytes32(0x79578ad1fa040c6ab66c0a551013f449b021a91fe3fce38e3f89240abe6021e6),
            ExitCode(SystemExitCode.Halted, 0),
            bytes32(0x0000000000000000000000000000000000000000000000000000000000000000),
            bytes32(0x420b84c1a220dc3bb1d61343217fbad879e5cfd72e224896384deb327305242c)
        )
    );

    bytes32 internal constant TEST_IMAGE_ID =
        bytes32(0xfb745ba9d74cce252c57ef0991a2e7cfc46304a1ece848cd98e5e57400f0707c);
    bytes internal constant TEST_JOURNAL =
        hex"5818100a2105c60d4f73044fe09a9cb0ba9801a4f5775e79cbb8934b23caab652d80a7843825f9cafc685d8307c8b06969e0f55bbec95ec79c8ca4131b3e29980000000105da591290223f1702e67293b817f5393e019ead000000019ace4afab142fbcbc90e317977a6800076bd64ba";

    RiscZeroVerifier internal verifier;

    function setUp() external {
        verifier = new RiscZeroVerifier();
    }

    function testVerifyKnownGoodReceipt() external view {
        require(verifier.verify(TEST_RECEIPT), "verification failed");
    }

    function testVerifyKnownGoodReceiptWithJournal() external view {
        require(verifier.verify(TEST_RECEIPT, TEST_JOURNAL), "verification failed");
    }

    function testVerifyKnownGoodImageIdAndJournal() external view {
        require(
            verifier.verify(TEST_RECEIPT.seal, TEST_IMAGE_ID, TEST_RECEIPT.meta.postStateDigest, TEST_JOURNAL),
            "verification failed"
        );
    }
}

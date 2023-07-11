// SPDX-License-Identifier: Apache-2.0
// TODO(victor): Determine what license this file needs to be.
pragma solidity ^0.8.9;

import {SafeCast} from "openzeppelin/contracts/utils/math/SafeCast.sol";

import {Groth16Verifier} from "./Groth16Verifier.sol";

import {console2} from "forge-std/console2.sol";
import {Vm} from "forge-std/Vm.sol";

/// @notice reverse the byte order of the uint256 value.
/// @dev Soldity uses a big-endian ABI encoding. Reversing the byte order before encoding
/// ensure that the encoded value will be little-endian.
/// Written by k06a. https://ethereum.stackexchange.com/a/83627
function reverseByteOrderUint256(uint256 input) pure returns (uint256 v) {
    v = input;

    // swap bytes
    v = ((v & 0xFF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00) >> 8)
        | ((v & 0x00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF) << 8);

    // swap 2-byte long pairs
    v = ((v & 0xFFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000) >> 16)
        | ((v & 0x0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF) << 16);

    // swap 4-byte long pairs
    v = ((v & 0xFFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000) >> 32)
        | ((v & 0x00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF) << 32);

    // swap 8-byte long pairs
    v = ((v & 0xFFFFFFFFFFFFFFFF0000000000000000FFFFFFFFFFFFFFFF0000000000000000) >> 64)
        | ((v & 0x0000000000000000FFFFFFFFFFFFFFFF0000000000000000FFFFFFFFFFFFFFFF) << 64);

    // swap 16-byte long pairs
    v = (v >> 128) | (v << 128);
}

/// @notice reverse the byte order of the uint32 value.
/// @dev Soldity uses a big-endian ABI encoding. Reversing the byte order before encoding
/// ensure that the encoded value will be little-endian.
/// Written by k06a. https://ethereum.stackexchange.com/a/83627
function reverseByteOrderUint32(uint32 input) pure returns (uint32 v) {
    v = input;

    // swap bytes
    v = ((v & 0xFF00FF00) >> 8) | ((v & 0x00FF00FF) << 8);

    // swap 2-byte long pairs
    v = (v >> 16) | (v << 16);
}

/// @notice Public state of a segment.
struct SystemState {
    /// The program counter.
    /// TODO(victor): What verification needs to be included for the initial pc.
    uint32 pc;
    /// @notice Root hash of a Merkle tree which confirms the
    /// integrity of the memory image.
    bytes32 merkleRoot;
}

library SystemStateLib {
    bytes32 constant TAG_DIGEST = sha256("risc0.SystemState");

    /// @notice Return the RISC Zero structural hash of the SystemState struct.
    function digest(SystemState memory state) internal pure returns (bytes32) {
        return sha256(abi.encodePacked(TAG_DIGEST, state.merkleRoot, reverseByteOrderUint32(state.pc), uint16(1) << 8));
    }
}

/// @notice Indicator for the overall system at the end of execution covered by this proof.
enum SystemExitCode {
    Halted,
    Paused,
    SystemSplit
}

/// @notice Combination of system and user exit codes.
/// @dev If system exit code is SystemSplit, the user exit code must be zero.
struct ExitCode {
    SystemExitCode system;
    uint8 user;
}

/// @notice Data associated with a receipt which is used for both input and
/// output of global state.
struct ReceiptMetadata {
    /// The SystemState of a segment just before execution has begun.
    SystemState pre;
    /// The SystemState of a segment just after execution has completed.
    SystemState post;
    /// The exit code for a segment
    ExitCode exitCode;
    /// A digest of the input, from the viewpoint of the guest.
    bytes32 input;
    /// A digest of the journal, from the viewpoint of the guest.
    bytes32 output;
}

library ReceiptMetadataLib {
    using SystemStateLib for SystemState;

    bytes32 constant TAG_DIGEST = sha256("risc0.ReceiptMeta");

    function digest(ReceiptMetadata memory meta) internal pure returns (bytes32) {
        // TODO(victor): Refactor the tagDigest to be a constant.
        return sha256(
            abi.encodePacked(
                TAG_DIGEST,
                // down
                meta.input,
                meta.pre.digest(),
                meta.post.digest(),
                meta.output,
                // data
                uint32(meta.exitCode.system) << 24,
                uint32(meta.exitCode.user) << 24,
                // down.length
                uint16(4) << 8
            )
        );
    }
}

/// @notice A Groth16 seal over the claimed receipt metadata.
struct Seal {
    uint256[2] a;
    uint256[2][2] b;
    uint256[2] c;
}

struct Receipt {
    Seal seal;
    ReceiptMetadata meta;
}

contract RiscZeroVerifier is Groth16Verifier {
    using SystemStateLib for SystemState;
    using ReceiptMetadataLib for ReceiptMetadata;
    using SafeCast for uint256;

    // Control ID hash for the identity_p254 predicate decomposed as implemented by splitDigest.
    uint256 internal constant CONTROL_ID_0 = uint256(0x1eece9585d11a13832b205d334d97478);
    uint256 internal constant CONTROL_ID_1 = uint256(0x06b74fed6685c71e0cf31d881093df86);

    /// @notice splits a digest into two 128-bit words to use as public signal inputs.
    /// @dev RISC Zero's Circom verifier circuit takes each of two hash digests in two 128-bit
    /// chunks. These values can be derived from the digest by splitting the digest in half and
    /// then reversing the bytes of each.
    function splitDigest(bytes32 digest) internal pure returns (uint256, uint256) {
        uint256 reversed = reverseByteOrderUint256(uint256(digest));
        return (uint256(uint128(uint256(reversed))), uint256(reversed >> 128));
    }

    /// @notice verify that the given receipt is a valid Groth16 RISC Zero recursion receipt.
    /// @return true if the receipt passes the verification checks.
    function verify(Receipt memory receipt) public view returns (bool) {
        bytes32 metadataDigest = receipt.meta.digest();
        (uint256 meta0, uint256 meta1) = splitDigest(metadataDigest);
        return
            this.verifyProof(receipt.seal.a, receipt.seal.b, receipt.seal.c, [CONTROL_ID_0, CONTROL_ID_1, meta0, meta1]);
    }

    /// @notice verify that the given receipt is a valid Groth16 RISC Zero recursion receipt with a
    ///     journal output equal to the given journal.
    /// @return true if the receipt passes the verification checks.
    function verify(Receipt memory receipt, bytes calldata journal) public view returns (bool) {
        // Verify that the Receipt's output field is equal to the journal hash.
        if (receipt.meta.output != sha256(journal)) {
            return false;
        }
        return verify(receipt);
    }

    /// @notice verifies that the given seal is a valid Groth16 RISC Zero proof of execution over the
    ///     given pre-state (i.e. image ID), post-state, and journal. Requires that the input hash
    //      is all-zeros (i.e. no committed input) and the exit code is (Halted, 0).
    /// @return true if the receipt passes the verification checks.
    function verify(Seal memory seal, SystemState memory pre, SystemState memory post, bytes32 imageId, bytes calldata journal)
        public
        view
        returns (bool)
    {
        // Check that the pre-state matches the given image ID.
        if (pre.imageId() != imageId) {
            console2.log(Vm(0x7109709ECfa91a80626fF3989D68f67F5b1DD12D).toString(pre.imageId()));
            console2.log(Vm(0x7109709ECfa91a80626fF3989D68f67F5b1DD12D).toString(imageId));
            revert("image ID mismatch");
            // return false;
        }
        Receipt memory receipt =
            Receipt(seal, ReceiptMetadata(pre, post, ExitCode(SystemExitCode.Halted, 0), bytes32(0), sha256(journal)));
        return verify(receipt);
    }
}

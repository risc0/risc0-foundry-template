// SPDX-License-Identifier: Apache-2.0
// TODO(victor): Determine what license this file needs to be.
pragma solidity ^0.8.9;

import {SafeCast} from "openzeppelin/contracts/utils/math/SafeCast.sol";

import {Groth16Verifier} from "./Groth16Verifier.sol";

/// @notice reverse the byte order of the uint32 value.
/// @dev Soldity uses a big-endian ABI encoding. Reversing the byte order before encoding
/// ensure that the encoded value will be little-endian.
/// Written by k06a. https://ethereum.stackexchange.com/a/83627
function reverseByteOrder(uint32 input) pure returns (uint32 v) {
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
        return sha256(abi.encodePacked(TAG_DIGEST, state.merkleRoot, reverseByteOrder(state.pc), uint16(1) << 8));
    }

    /// @notice Return the RISC Zero image ID computed from the memory root and program counter.
    function imageId(SystemState memory state) internal pure returns (bytes32) {
        return sha256(abi.encodePacked(state.merkleRoot, uint256(reverseByteOrder(state.pc))));
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
    /// TODO(victor): Does the input ever get set?
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
    bytes32 controlId;
    ReceiptMetadata meta;
}

contract RiscZeroVerifier is Groth16Verifier {
    using ReceiptMetadataLib for ReceiptMetadata;
    using SafeCast for uint256;

    // @notice returns true if the given digest is a member of the set of control IDs for valid recursion predicates.
    // @dev This list must be kept up to date with the current revision of of the RISC Zero recursion circuit and predicates.
    function isValidControlId(bytes32 id) internal pure returns (bool) {
        // Just compare and OR with all 29 possible values. Costs a constant 173 gas.
        return id == bytes32(0x89857430b8b5872251f0b9342dd1eb326767e35af5db2e3e05cb95692f716c06)
            || id == bytes32(0xa2408a494714ff38b26cb419c295b9014db6f951628f7333d0e3a12d1ff88f39)
            || id == bytes32(0x0bdfb159b1d8e91f0cb29936fd4647134bf4466f4e195a36f9cb7d32004d0e0c)
            || id == bytes32(0x5c10b043948b8b6165e9f659d496c10b5710124c8483a727d83734381888ec2d)
            || id == bytes32(0x6cc844708f26060c859bf8369d62042bc40a4a39a1d45556b046ac710c113a44)
            || id == bytes32(0x201a022b0c00f269a2642954d9bc50600fccd261d9a5f14ad6e512386edc6a40)
            || id == bytes32(0x60465e0b678fa2231e6f4f74ea6786497eae7019ff746158c57d563ae9e5e015)
            || id == bytes32(0xba9aa10592a94a582e081d4180a83b70ad83e552c055081e4704f95f3132903b)
            || id == bytes32(0x5e2afd43e98ce42ae1e45b21b7816362ea07c14b451e586ebc3abc61d7927963)
            || id == bytes32(0x163238734416f738562cc56eaf341643ac95815a5aa00771fa0ee109e5c8521f)
            || id == bytes32(0x0090fe6d3e5c57563bab8a6417396834c281341de3ad14267e62a275eb63742a)
            || id == bytes32(0xa3372a0f17ec1f325398ac292c04491f5c23502ac6a58e3d2340730d83847023)
            || id == bytes32(0x69b7401da9f7cb60245bf06a869c1651af26844a89e6b353fd21ce2f2f831339)
            || id == bytes32(0xf3de611d2dcf6513c80f1917b02234500422166318c6635b0c4e260053d73467)
            || id == bytes32(0xb559182374741a2d2d684d41f12ab534b0496222aef9f160850687474d066a3d)
            || id == bytes32(0xb5061e48ce0cb04823f9f63810838460d71aad3cda52df77fa8f7402ccb0e132)
            || id == bytes32(0x85bdb1317ba858651b30114bb9941f0af456fb2a28ab1f5a338dfa4d37e47e69)
            || id == bytes32(0x542ed0378fad6368e764ef3b8b8da543a876d46b2b8a8f5c739a69437552bd40)
            || id == bytes32(0xfb8fd507b2717d6f73843d16053b90087ededb3a4e851a771b8857589af43369)
            || id == bytes32(0x557a443f21ff0d15174c2849a9a4f618815d561414d7e9490f14b7208086cb6a)
            || id == bytes32(0xd441e538b20dd60a7d81de2e4a731f6493bc7b4c029093336f0b4117dc6f574a)
            || id == bytes32(0x88ac0d189774a6439abce033047f620c62379d66d52e18425c4e2705b2b5c613)
            || id == bytes32(0x84a0a81fc34d6d631d91aa4d80aa38525d733c56cba506175f9b94540627ef00)
            || id == bytes32(0xb8445500b4838e3bdca70d3906c1cc711ce03c325094152743e4da5cedaaaa05)
            || id == bytes32(0xb4ef2c6628e8c81f900b311a4f1bde46918f062f18a01064bdb274229175c46b)
            || id == bytes32(0xb536a45fae0e2370e71aeb567f55452a791c63636bb4195c6b456003554c386d)
            || id == bytes32(0x6adfed538d2c084a6615755acc9bb8333126546b5426ad5b34f97713f6a4d269)
            || id == bytes32(0x93ddfd13ad326c73317033379bda035dd1e97a056aaed01c5beca909d764d213)
            || id == bytes32(0x2f3d304a4c80e02bee2ba61a9ce27812d03a4e41acf9fd4eb563d3687e2dd859);
    }

    /// @notice splits a digest into two 128-bit words to use as public signal inputs.
    function splitDigest(bytes32 digest) internal pure returns (uint256, uint256) {
        return (uint256(digest >> 128), uint256(uint128(uint256(digest))));
    }

    function verify(Receipt memory receipt) public view returns (bool) {
        require(isValidControlId(receipt.controlId), "RiscZeroVerifier: controlId is invalid");
        bytes32 metadataDigest = receipt.meta.digest();
        // TODO(victor): This is mostly a guess.
        (uint256 metaHi, uint256 metaLo) = splitDigest(metadataDigest);
        (uint256 controlHi, uint256 controlLo) = splitDigest(receipt.controlId);
        return this.verifyProof(receipt.seal.a, receipt.seal.b, receipt.seal.c, [metaLo, metaHi, controlLo, controlHi]);
    }
}

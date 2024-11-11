// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./MerkleTreeWithHistory.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {ImageID} from "./ImageID.sol"; // auto-generated contract after running `cargo build`.
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

abstract contract Mixer is MerkleTreeWithHistory, ReentrancyGuard {
    IRiscZeroVerifier public immutable verifier;
    bytes32 public constant imageId = ImageID.CAN_SPEND_ID;

    uint256 public denomination;

    mapping(bytes32 => bool) public nullifierHashes;
    // we store all commitments just to prevent accidental deposits with the same commitment
    mapping(bytes32 => bool) public commitments;

    event Deposit(
        bytes32 indexed commitment,
        uint32 leafIndex,
        uint256 timestamp
    );
    event Withdrawal(address to, bytes32 nullifierHash);

    /**
    @dev The constructor
    @param _verifier the address of SNARK verifier for this contract
    @param _denomination transfer amount for each deposit
    @param _merkleTreeHeight the height of deposits' Merkle Tree
  */
    constructor(
        IRiscZeroVerifier _verifier,
        uint256 _denomination,
        uint32 _merkleTreeHeight
    ) MerkleTreeWithHistory(_merkleTreeHeight) {
        require(_denomination > 0, "denomination should be greater than 0");
        verifier = _verifier;
        denomination = _denomination;
    }

    /**
    @dev Deposit funds into the contract. The caller must send (for ETH) or approve (for ERC20) value equal to or `denomination` of this instance.
    @param _commitment the note commitment, which is Sha256(nullifier || secret)
  */
    function deposit(bytes32 _commitment) external payable nonReentrant {
        require(!commitments[_commitment], "The commitment has been submitted");

        uint32 insertedIndex = _insert(_commitment);
        commitments[_commitment] = true;
        _processDeposit();

        emit Deposit(_commitment, insertedIndex, block.timestamp);
    }

    /** @dev this function is defined in a child contract */
    function _processDeposit() internal virtual;

    /**
    @dev Withdraw a deposit from the contract. `proof` is a zkSNARK proof data, and input is an array of circuit public inputs
    `input` array consists of:
      - merkle root of all deposits in the contract
      - hash of unique deposit nullifier to prevent double spends
  */
    function withdraw(
        bytes calldata _seal,
        bytes32 _root,
        bytes32 _nullifierHash
    ) external payable nonReentrant {
        require(
            !nullifierHashes[_nullifierHash],
            "The note has been already spent"
        );
        require(isKnownRoot(_root), "Cannot find your merkle root"); // Make sure to use a recent one

        // Verify the proof and ensure the journal matches
        bytes memory journal = abi.encodePacked(
            _root,
            _nullifierHash,
            msg.sender
        );
        verifier.verify(_seal, imageId, sha256(journal));

        nullifierHashes[_nullifierHash] = true;
        _processWithdraw(payable(msg.sender));
        emit Withdrawal(msg.sender, _nullifierHash);
    }

    /** @dev this function is defined in a child contract */
    function _processWithdraw(address payable _recipient) internal virtual;

    /** @dev whether a note is already spent */
    function isSpent(bytes32 _nullifierHash) public view returns (bool) {
        return nullifierHashes[_nullifierHash];
    }

    /** @dev whether an array of notes is already spent */
    function isSpentArray(
        bytes32[] calldata _nullifierHashes
    ) external view returns (bool[] memory spent) {
        spent = new bool[](_nullifierHashes.length);
        for (uint256 i = 0; i < _nullifierHashes.length; i++) {
            if (isSpent(_nullifierHashes[i])) {
                spent[i] = true;
            }
        }
    }
}

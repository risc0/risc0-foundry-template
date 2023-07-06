// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "openzeppelin/contracts/governance/IGovernor.sol";
import "openzeppelin/contracts/governance/Governor.sol";

abstract contract IBonsaiGovernor is IGovernor, Governor {
    // Copied from the Governor contract.
    /// @notice Calculate the message digest to sign in order to call castVoteBySig.
    function voteHash(uint256 proposalId, uint8 support) public view virtual returns (bytes32) {
        return _hashTypedDataV4(keccak256(abi.encode(BALLOT_TYPEHASH, proposalId, support)));
    }

    /// @notice Calculate the message digest to sign in order to call castVoteBySig.
    function voteHashWithReasonAndParamsBySig(
        uint256 proposalId,
        uint8 support,
        string calldata reason,
        bytes memory params
    ) public view virtual returns (bytes32) {
        return _hashTypedDataV4(
            keccak256(
                abi.encode(EXTENDED_BALLOT_TYPEHASH, proposalId, support, keccak256(bytes(reason)), keccak256(params))
            )
        );
    }

    /// @dev Optional. If not implemented, this function is a no-op and it is assumed it does not
    /// need to be called for the final vote count to be available.
    function finalizeVotes(uint256 proposalId, bytes32 finalBallotBoxAccum, bytes24[] calldata ballots)
        external
        virtual
    {}
}

// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "openzeppelin/contracts/governance/Governor.sol";
import "openzeppelin/contracts/governance/extensions/GovernorSettings.sol";
import "openzeppelin/contracts/governance/extensions/GovernorVotes.sol";
import "openzeppelin/contracts/governance/extensions/GovernorVotesQuorumFraction.sol";

import "./IBonsaiGovernor.sol";
import "./BonsaiGovernorCounting.sol";

/// @custom:security-contact security@risczero.com
contract BonsaiGovernor is
    IBonsaiGovernor,
    GovernorSettings,
    BonsaiGovernorCounting,
    GovernorVotes,
    GovernorVotesQuorumFraction
{
    constructor(IVotes _token)
        Governor("BonsaiGovernor")
        GovernorSettings(300, /* blocks */ 21000, /* blocks */ 0)
        GovernorVotes(_token)
        GovernorVotesQuorumFraction(20)
    {}

    /**
     * @notice Calculate the current state of the proposal.
     * @dev See {IGovernor-state}.
     */
    function state(uint256 proposalId) public view override(IGovernor, Governor) returns (ProposalState) {
        ProposalState superState = super.state(proposalId);

        // If the votes have not been finalized, by proving the off-chain verified list of validated
        // ballots, then keep the proposal status as active. IGovernor does not provide a state to
        // indicate that voting has ended, but the result is unknown.
        if (superState == ProposalState.Defeated && !_proposalVotesFinalized(proposalId)) {
            return ProposalState.Active;
        }
        return superState;
    }

    function finalizeVotes(uint256 proposalId, bytes32 finalBallotBoxAccum, bytes24[] calldata ballots)
        external
        override
    {
        _finalizeVotes(proposalId, finalBallotBoxAccum, ballots);
    }

    // TODO(victor): What are the effects of not being able to return the voting weight from
    //               castVoteBySig functions.

    /**
     * @dev See {IGovernor-castVote}.
     *      Does not return the voter's balance, since balance lookups are deferred.
     */
    function castVote(uint256 proposalId, uint8 support) public override(Governor, IGovernor) returns (uint256) {
        address voter = _msgSender();
        _commitVote(proposalId, support, voter);
        emit VoteCast(voter, proposalId, support, 0, "");
        return 0;
    }

    /**
     * @dev See {IGovernor-castVoteWithReason}.
     *      Does not return the voter's balance, since balance lookups are deferred.
     */
    function castVoteWithReason(uint256 proposalId, uint8 support, string calldata reason)
        public
        override(Governor, IGovernor)
        returns (uint256)
    {
        address voter = _msgSender();
        _commitVote(proposalId, support, voter);
        emit VoteCast(voter, proposalId, support, 0, reason);
        return 0;
    }

    /*
     * @dev See {IGovernor-castVoteWithReasonAndParams}.
     *      Does not return the voter's balance, since balance lookups are deferred.
     */
    function castVoteWithReasonAndParams(uint256 proposalId, uint8 support, string calldata reason, bytes memory params)
        public
        override(Governor, IGovernor)
        returns (uint256)
    {
        require(params.length == 0, "BonsaiGovernor: params are not supported");

        address voter = _msgSender();
        _commitVote(proposalId, support, voter);
        emit VoteCast(voter, proposalId, support, 0, reason);
        return 0;
    }

    /**
     * @dev See {IGovernor-castVoteBySig}.
     *      Does not return the voter's balance, since balance lookups are deferred.
     *      Also does not log a VoteCast event because it cannot be determined yet if this is a valid vote.
     */
    function castVoteBySig(uint256 proposalId, uint8 support, uint8 v, bytes32 r, bytes32 s)
        public
        override(Governor, IGovernor)
        returns (uint256)
    {
        bytes32 digest = voteHash(proposalId, support);
        _commitVoteBySig(proposalId, support, v, r, s, digest);
        return 0;
    }

    /**
     * @dev See {IGovernor-castVoteWithReasonAndParamsBySig}.
     *      Does not return the voter's balance, since balance lookups are deferred.
     *      Also does not log a VoteCast event because it cannot be determined yet if this is a valid vote.
     */
    function castVoteWithReasonAndParamsBySig(
        uint256 proposalId,
        uint8 support,
        string calldata reason,
        bytes memory params,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) public override(Governor, IGovernor) returns (uint256) {
        require(params.length == 0, "BonsaiGovernor: params are not supported");

        bytes32 digest = voteHashWithReasonAndParamsBySig(proposalId, support, reason, params);
        _commitVoteBySig(proposalId, support, v, r, s, digest);
        return 0;
    }

    function _castVote(uint256, address, uint8, string memory, bytes memory) internal pure override returns (uint256) {
        revert("_castVote is not supported");
    }

    // The following functions are overrides required by Solidity.

    function propose(
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        string memory description
    ) public override(IGovernor, Governor, BonsaiGovernorCounting) returns (uint256) {
        return super.propose(targets, values, calldatas, description);
    }

    function votingDelay() public view override(IGovernor, GovernorSettings) returns (uint256) {
        return super.votingDelay();
    }

    function votingPeriod() public view override(IGovernor, GovernorSettings) returns (uint256) {
        return super.votingPeriod();
    }

    function quorum(uint256 blockNumber)
        public
        view
        override(IGovernor, GovernorVotesQuorumFraction)
        returns (uint256)
    {
        return super.quorum(blockNumber);
    }

    function proposalThreshold() public view override(Governor, GovernorSettings) returns (uint256) {
        return super.proposalThreshold();
    }
}

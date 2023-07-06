// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "openzeppelin/contracts/utils/Strings.sol";
import "openzeppelin/contracts/governance/extensions/GovernorCountingSimple.sol";
import "openzeppelin/contracts/governance/IGovernor.sol";
import "openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "openzeppelin/contracts/utils/math/SafeMath.sol";

import "forge-std/Vm.sol";
import "forge-std/Test.sol";
import "forge-std/console2.sol";
import "solidity-bytes-utils//BytesLib.sol";

import "../contracts/BonsaiGovernor.sol";
import "../contracts/BaselineGovernor.sol";
import "../contracts/IBonsaiGovernor.sol";
import "../contracts/VoteToken.sol";

/// @notice Voter to be included in a test scenario.
contract Voter is Test {
    IBonsaiGovernor internal gov;
    VoteToken internal token;

    /// @notice whether the account voting power is delegated to an EOA.
    bool public eoa;
    /// @notice voting weight of the voter. equal to token balance.
    uint256 public weight;

    // Copied from IGovernor to set up vm.expectEmit.
    event VoteCast(address indexed voter, uint256 proposalId, uint8 support, uint256 weight, string reason);

    /// @notice create a new voter.
    constructor(IBonsaiGovernor gov_, VoteToken token_, bool eoa_, uint256 weight_) {
        gov = gov_;
        token = token_;
        eoa = eoa_;
        weight = weight_;

        // Mint and delegate tokens equal to the weight.
        vm.prank(token.owner());
        token.mint(address(this), weight);
        delegate();
    }

    /// @notice returns the private key to use for signing votes.
    function delegateKey() public view returns (uint256) {
        require(eoa, "only eoa voters have a private key");
        return uint256(uint160(address(this)));
    }

    /// @notice returns the delegated voting address.
    function delegateAddr() public view returns (address) {
        if (eoa) {
            return vm.addr(delegateKey());
        } else {
            return address(this);
        }
    }

    /// @notice delegates the voting power of this voter to its delegate address.
    function delegate() public {
        token.delegate(delegateAddr());
    }

    function vote(uint256 proposalId, uint8 support) public {
        // Event data may not match because BonsaiGovernor does not resolve voter weight right away.
        vm.prank(delegateAddr()); // NOTE: Only needed for EOAs, but always works.
        vm.expectEmit(true, false, false, false, address(gov));
        emit VoteCast(delegateAddr(), proposalId, support, uint256(0), "");
        gov.castVote(proposalId, support);
    }

    function voteBySig(uint256 proposalId, uint8 support) public {
        require(eoa, "only eoa voters have a private key");
        bytes32 digest = gov.voteHash(proposalId, support);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(delegateKey(), digest);

        // Does not check that the VoteCast event is emitted becasue BonsaiGovernor does not emit this event.
        gov.castVoteBySig(proposalId, support, v, r, s);
    }
}

struct Vote {
    /// @notice voter to be casting the vote.
    Voter voter;
    /// @notice whether to vote by signature or direct call.
    bool signed;
    /// @notice support (e.g. For, Against, Abstain) for the vote.
    GovernorCountingSimple.VoteType support;
}

library VoteLib {
    function cast(Vote memory vote, uint256 proposalId) internal {
        if (vote.signed) {
            vote.voter.voteBySig(proposalId, uint8(vote.support));
        } else {
            vote.voter.vote(proposalId, uint8(vote.support));
        }
    }
}

contract Scenario {
    using SafeMath for uint256;
    using VoteLib for Vote;

    IBonsaiGovernor internal gov;
    VoteToken internal token;

    /// @notice list of voters to register and cast votes from.
    Voter[] public voters;
    /// @notice list of votes to be carried out by the voters, in order.
    Vote[] public votes;
    /// @notice indicator for whether or not the proposal should pass.
    bool public success;

    constructor(IBonsaiGovernor gov_, VoteToken token_, bool success_) {
        gov = gov_;
        token = token_;
        success = success_;
    }

    function addVoter(bool eoa, uint256 weight) public returns (Voter) {
        Voter voter = new Voter(gov, token, eoa, weight);
        voters.push(voter);
        return voter;
    }

    function addVote(Voter voter, bool signed, GovernorCountingSimple.VoteType support) public {
        votes.push(Vote(voter, signed, support));
    }

    function castVotes(uint256 proposalId) public {
        for (uint256 i = 0; i < votes.length; i = i.add(1)) {
            votes[i].cast(proposalId);
        }
    }
}

abstract contract GovernorTest is Test {
    using SafeMath for uint256;
    using VoteLib for Vote;

    event ProposalCallbackCalled();

    IBonsaiGovernor internal gov;
    VoteToken internal token;
    Scenario internal scene;

    string constant PROPOSAL_DESC = "test proposal description";

    function governor(VoteToken token_) internal virtual returns (IBonsaiGovernor);

    function scenario(IBonsaiGovernor gov_, VoteToken token_) internal virtual returns (Scenario);

    function finalizeVotes(uint256 proposalId) internal virtual;

    // TODO(victor): Write up a variety of test cases for different voting combinations and whatnot.
    // * Many votes
    // * Failing to reach consensus
    // * Failing to each quorum
    // * Repeated votes
    //
    // Measure the gas consumption as it scales with number of votes.

    function proposalCallback() external {
        assertTrue(scene.success());
        emit ProposalCallbackCalled();
    }

    function propose() internal returns (uint256) {
        // Assemble a simple test proposal to call this contract.
        address[] memory targets = new address[](1);
        uint256[] memory values = new uint256[](1);
        bytes[] memory calldatas = new bytes[](1);

        targets[0] = address(this);
        values[0] = uint256(0);
        calldatas[0] = abi.encodeWithSelector(this.proposalCallback.selector);

        return gov.propose(targets, values, calldatas, PROPOSAL_DESC);
    }

    function execute() internal {
        // Re-assemble the test proposal.
        address[] memory targets = new address[](1);
        uint256[] memory values = new uint256[](1);
        bytes[] memory calldatas = new bytes[](1);

        targets[0] = address(this);
        values[0] = uint256(0);
        calldatas[0] = abi.encodeWithSelector(this.proposalCallback.selector);

        if (scene.success()) {
            vm.expectEmit(address(this));
            emit ProposalCallbackCalled();
        } else {
            vm.expectRevert(bytes("Governor: proposal not successful"));
        }
        gov.execute(targets, values, calldatas, keccak256(bytes(PROPOSAL_DESC)));
    }

    function testPropose() external {
        propose();
    }

    function testVote() external {
        uint256 proposalId = propose();

        vm.roll(block.number + gov.votingDelay() + 1);
        scene.castVotes(proposalId);
    }

    function testExecute() external {
        uint256 proposalId = propose();

        vm.roll(block.number + gov.votingDelay() + 1);
        scene.castVotes(proposalId);

        vm.roll(gov.proposalDeadline(proposalId) + 1);
        finalizeVotes(proposalId);
        execute();
    }
}

abstract contract BaselineGovernorTest is GovernorTest {
    function governor(VoteToken token) internal override returns (IBonsaiGovernor) {
        return new BaselineGovernor(token);
    }

    function finalizeVotes(uint256 proposalId) internal override {}

    function setUp() public {
        token = new VoteToken();
        gov = governor(token);
        scene = scenario(gov, token);
    }
}

abstract contract BonsaiGovernorTest is GovernorTest {
    using SafeMath for uint256;
    using BytesLib for bytes;
    using VoteLib for Vote;

    // Copied from BonsaiGovernorCounting
    event CommittedBallot(uint256 indexed proposalId, bytes encoded);

    struct BallotBox {
        bytes32 commit;
        mapping(address => bool) hasVoted;
        mapping(address => uint8) support;
        address[] voters;
    }

    /// @notice mapping of proposals to ballot boxes.
    /// @dev ballots are persisted to storage because evnts can only ever be obtained once from vm.getRecordedLogs().
    mapping(uint256 => BallotBox) ballotBoxes;

    function setUp() public {
        token = new VoteToken();
        gov = governor(token);
        scene = scenario(gov, token);

        // Enable recording of logs so we can build the ballot list.
        vm.recordLogs();
    }

    function governor(VoteToken token) internal override returns (IBonsaiGovernor) {
        return new BonsaiGovernor(token);
    }

    /// @notice collect the ballots and reconstruct ballot box commit by iterating through events.
    /// @dev this function mocks what the zkVM guest will do.
    function collectBallots(uint256 proposalId) internal returns (bytes32, bytes24[] memory) {
        // This function normally executes off-chain in the guest.
        vm.pauseGasMetering();

        BallotBox storage box = ballotBoxes[proposalId];
        if (box.commit == bytes32(0)) {
            box.commit = bytes32(proposalId);
        }

        // Retrieve the recorded events. Note that this consumes them.
        Vm.Log[] memory entries = vm.getRecordedLogs();
        for (uint256 i = 0; i < entries.length; i = i.add(1)) {
            Vm.Log memory entry = entries[i];
            if (entry.topics[0] != CommittedBallot.selector) {
                continue;
            }
            require(uint256(entry.topics[1]) == proposalId, "proposal id mismatch in event");
            bytes memory encodedBallot = abi.decode(entry.data, (bytes));
            box.commit = sha256(bytes.concat(box.commit, encodedBallot));

            // Decode the custom encoding format for ballots.
            // TODO(victor): Use a more standard encoding format?
            require(encodedBallot[0] == bytes1(0), "upper byte of signed is non-zero");
            uint8 signed = uint8(encodedBallot[1]);
            uint8 support = uint8(encodedBallot[2]);
            address voter;
            if (signed == uint8(1)) {
                // Decode a ballot with an attached signature.
                require(encodedBallot.length == uint256(100), "encoded ballot w signature must be 100 bytes");
                uint8 v = uint8(encodedBallot[3]);
                (bytes32 r, bytes32 s, bytes32 sigDigest) =
                    abi.decode(encodedBallot.slice(4, 96), (bytes32, bytes32, bytes32));

                // NOTE: It is almost never safe to "verify" a signature on a provided digest.
                // Here we guarantee that the hashing in this context what was observed on-chain through the ballot box commitments.
                voter = ECDSA.recover(sigDigest, v, r, s);
            } else {
                // Decode a ballot with no attached signature.
                require(encodedBallot.length == uint256(24), "encoded ballot w/o signature must be 24 bytes");
                require(signed == uint16(0), "value of signed is not boolean");
                require(encodedBallot[3] == bytes1(0), "padding bytes is non-zero");
                voter = encodedBallot.toAddress(4);
            }

            // If someone votes twice, we allow it by updating their vote.
            if (!box.hasVoted[voter]) {
                box.voters.push(voter);
            }
            box.hasVoted[voter] = true;
            box.support[voter] = support;
        }

        bytes24[] memory ballots = new bytes24[](box.voters.length);
        for (uint256 i = 0; i < box.voters.length; i = i.add(1)) {
            address voter = box.voters[i];
            uint8 support = box.support[voter];

            // Encode the address and support to 24 bytes and push it to the ballots array.
            ballots[i] = bytes24((uint192(support) << 160) | uint192(uint160(voter)));
        }

        vm.resumeGasMetering();
        return (box.commit, ballots);
    }

    function finalizeVotes(uint256 proposalId) internal override {
        (bytes32 commit, bytes24[] memory ballots) = collectBallots(proposalId);
        gov.finalizeVotes(proposalId, commit, ballots);
    }

    function testFinalize() public {
        uint256 proposalId = propose();

        vm.roll(block.number + gov.votingDelay() + 1);
        scene.castVotes(proposalId);

        (bytes32 commit, bytes24[] memory ballots) = collectBallots(proposalId);

        // Finalize can only be called after voting concludes.
        vm.expectRevert();
        gov.finalizeVotes(proposalId, commit, ballots);

        // Move the block number forward past the voting deadline.
        vm.roll(gov.proposalDeadline(proposalId) + 1);

        // Check that before finalization, the state is active and after it is success.
        require(gov.state(proposalId) == IGovernor.ProposalState.Active, "expected proposal state active");

        gov.finalizeVotes(proposalId, commit, ballots);
        if (scene.success()) {
            require(gov.state(proposalId) == IGovernor.ProposalState.Succeeded, "expected proposal state Succeeded");
        } else {
            require(gov.state(proposalId) == IGovernor.ProposalState.Defeated, "expected proposal state Defeated");
        }

        // Finalize can only be called once.
        vm.expectRevert();
        gov.finalizeVotes(proposalId, commit, ballots);
    }
}

abstract contract BasicTest is GovernorTest {
    function scenario(IBonsaiGovernor gov, VoteToken token) internal override returns (Scenario) {
        scene = new Scenario(gov, token, true);

        Voter voter;
        voter = scene.addVoter(false, 50);
        scene.addVote(voter, false, GovernorCountingSimple.VoteType.For);
        voter = scene.addVoter(true, 50);
        scene.addVote(voter, true, GovernorCountingSimple.VoteType.For);

        return scene;
    }
}

abstract contract BasicFailingTest is GovernorTest {
    function scenario(IBonsaiGovernor gov, VoteToken token) internal override returns (Scenario) {
        scene = new Scenario(gov, token, false);

        Voter voter;
        voter = scene.addVoter(false, 50);
        scene.addVote(voter, false, GovernorCountingSimple.VoteType.For);
        voter = scene.addVoter(true, 50);
        scene.addVote(voter, true, GovernorCountingSimple.VoteType.Against);

        return scene;
    }
}

abstract contract BenchTest is GovernorTest {
    using SafeMath for uint256;

    uint256 internal voteCount;

    constructor(uint256 voteCount_) {
        voteCount = voteCount_;
    }

    function scenario(IBonsaiGovernor gov, VoteToken token) internal override returns (Scenario) {
        scene = new Scenario(gov, token, true);

        Voter voter;
        for (uint256 i = 0; i < voteCount; i = i.add(1)) {
            voter = scene.addVoter(true, 10);
            scene.addVote(voter, true, GovernorCountingSimple.VoteType.For);
        }

        return scene;
    }
}

contract BasicBaselineGovernorTest is BaselineGovernorTest, BasicTest {}

contract BasicBonsaiGovernorTest is BonsaiGovernorTest, BasicTest {}

contract BasicFailingBaselineGovernorTest is BaselineGovernorTest, BasicFailingTest {}

contract BasicFailingBonsaiGovernorTest is BonsaiGovernorTest, BasicFailingTest {}

contract BenchBaselineTest is BaselineGovernorTest, BenchTest {
    constructor() BenchTest(100) {}
}

contract BenchBonsaiTest is BonsaiGovernorTest, BenchTest {
    constructor() BenchTest(100) {}
}


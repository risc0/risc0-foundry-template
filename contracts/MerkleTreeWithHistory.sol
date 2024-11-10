// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract MerkleTreeWithHistory {
    uint32 public levels;

    // the following variables are made public for easier testing and debugging and
    // are not supposed to be accessed in regular code

    // filledSubtrees and roots could be bytes32[size], but using mappings makes it cheaper because
    // it removes index range check on every interaction
    mapping(uint256 => bytes32) public filledSubtrees;
    mapping(uint256 => bytes32) public roots;
    uint32 public constant ROOT_HISTORY_SIZE = 30;
    uint32 public currentRootIndex = 0;
    uint32 public nextIndex = 0;

    constructor(uint32 _levels) {
        require(_levels > 0, "_levels should be greater than zero");
        require(_levels < 32, "_levels should be less than 32");
        levels = _levels;

        for (uint32 i = 0; i < _levels; i++) {
            filledSubtrees[i] = zeros(i);
        }

        roots[0] = zeros(_levels - 1);
    }

    /**
    @dev Hash 2 tree leaves, returns sha256(_left||_right)
  */
    function hashLeftRight(
        bytes32 _left,
        bytes32 _right
    ) public pure returns (bytes32) {
        return sha256(abi.encodePacked(_left, _right));
    }

    function _insert(bytes32 _leaf) internal returns (uint32 index) {
        uint32 _nextIndex = nextIndex;
        require(
            _nextIndex != uint32(2) ** levels,
            "Merkle tree is full. No more leaves can be added"
        );
        uint32 currentIndex = _nextIndex;
        bytes32 currentLevelHash = _leaf;
        bytes32 left;
        bytes32 right;

        for (uint32 i = 0; i < levels; i++) {
            if (currentIndex % 2 == 0) {
                left = currentLevelHash;
                right = zeros(i);
                filledSubtrees[i] = currentLevelHash;
            } else {
                left = filledSubtrees[i];
                right = currentLevelHash;
            }
            currentLevelHash = hashLeftRight(left, right);
            currentIndex /= 2;
        }

        uint32 newRootIndex = (currentRootIndex + 1) % ROOT_HISTORY_SIZE;
        currentRootIndex = newRootIndex;
        roots[newRootIndex] = currentLevelHash;
        nextIndex = _nextIndex + 1;
        return _nextIndex;
    }

    /**
    @dev Whether the root is present in the root history
  */
    function isKnownRoot(bytes32 _root) public view returns (bool) {
        if (_root == 0) {
            return false;
        }
        uint32 _currentRootIndex = currentRootIndex;
        uint32 i = _currentRootIndex;
        do {
            if (_root == roots[i]) {
                return true;
            }
            if (i == 0) {
                i = ROOT_HISTORY_SIZE;
            }
            i--;
        } while (i != _currentRootIndex);
        return false;
    }

    /**
    @dev Returns the last root
  */
    function getLastRoot() public view returns (bytes32) {
        return roots[currentRootIndex];
    }

    /// @dev provides Zero (Empty) elements for a sha2 MerkleTree. Up to 32 levels
    function zeros(uint256 i) public pure returns (bytes32) {
        if (i == 0)
            return
                bytes32(
                    0x0000000000000000000000000000000000000000000000000000000000000000
                );
        else if (i == 1)
            return
                bytes32(
                    0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b
                );
        else if (i == 2)
            return
                bytes32(
                    0xdb56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71
                );
        else if (i == 3)
            return
                bytes32(
                    0xc78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c
                );
        else if (i == 4)
            return
                bytes32(
                    0x536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c
                );
        else if (i == 5)
            return
                bytes32(
                    0x9efde052aa15429fae05bad4d0b1d7c64da64d03d7a1854a588c2cb8430c0d30
                );
        else if (i == 6)
            return
                bytes32(
                    0xd88ddfeed400a8755596b21942c1497e114c302e6118290f91e6772976041fa1
                );
        else if (i == 7)
            return
                bytes32(
                    0x87eb0ddba57e35f6d286673802a4af5975e22506c7cf4c64bb6be5ee11527f2c
                );
        else if (i == 8)
            return
                bytes32(
                    0x26846476fd5fc54a5d43385167c95144f2643f533cc85bb9d16b782f8d7db193
                );
        else if (i == 9)
            return
                bytes32(
                    0x506d86582d252405b840018792cad2bf1259f1ef5aa5f887e13cb2f0094f51e1
                );
        else if (i == 10)
            return
                bytes32(
                    0xffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b
                );
        else if (i == 11)
            return
                bytes32(
                    0x6cf04127db05441cd833107a52be852868890e4317e6a02ab47683aa75964220
                );
        else if (i == 12)
            return
                bytes32(
                    0xb7d05f875f140027ef5118a2247bbb84ce8f2f0f1123623085daf7960c329f5f
                );
        else if (i == 13)
            return
                bytes32(
                    0xdf6af5f5bbdb6be9ef8aa618e4bf8073960867171e29676f8b284dea6a08a85e
                );
        else if (i == 14)
            return
                bytes32(
                    0xb58d900f5e182e3c50ef74969ea16c7726c549757cc23523c369587da7293784
                );
        else if (i == 15)
            return
                bytes32(
                    0xd49a7502ffcfb0340b1d7885688500ca308161a7f96b62df9d083b71fcc8f2bb
                );
        else if (i == 16)
            return
                bytes32(
                    0x8fe6b1689256c0d385f42f5bbe2027a22c1996e110ba97c171d3e5948de92beb
                );
        else if (i == 17)
            return
                bytes32(
                    0x8d0d63c39ebade8509e0ae3c9c3876fb5fa112be18f905ecacfecb92057603ab
                );
        else if (i == 18)
            return
                bytes32(
                    0x95eec8b2e541cad4e91de38385f2e046619f54496c2382cb6cacd5b98c26f5a4
                );
        else if (i == 19)
            return
                bytes32(
                    0xf893e908917775b62bff23294dbbe3a1cd8e6cc1c35b4801887b646a6f81f17f
                );
        else if (i == 20)
            return
                bytes32(
                    0xcddba7b592e3133393c16194fac7431abf2f5485ed711db282183c819e08ebaa
                );
        else if (i == 21)
            return
                bytes32(
                    0x8a8d7fe3af8caa085a7639a832001457dfb9128a8061142ad0335629ff23ff9c
                );
        else if (i == 22)
            return
                bytes32(
                    0xfeb3c337d7a51a6fbf00b9e34c52e1c9195c969bd4e7a0bfd51d5c5bed9c1167
                );
        else if (i == 23)
            return
                bytes32(
                    0xe71f0aa83cc32edfbefa9f4d3e0174ca85182eec9f3a09f6a6c0df6377a510d7
                );
        else if (i == 24)
            return
                bytes32(
                    0x31206fa80a50bb6abe29085058f16212212a60eec8f049fecb92d8c8e0a84bc0
                );
        else if (i == 25)
            return
                bytes32(
                    0x21352bfecbeddde993839f614c3dac0a3ee37543f9b412b16199dc158e23b544
                );
        else if (i == 26)
            return
                bytes32(
                    0x619e312724bb6d7c3153ed9de791d764a366b389af13c58bf8a8d90481a46765
                );
        else if (i == 27)
            return
                bytes32(
                    0x7cdd2986268250628d0c10e385c58c6191e6fbe05191bcc04f133f2cea72c1c4
                );
        else if (i == 28)
            return
                bytes32(
                    0x848930bd7ba8cac54661072113fb278869e07bb8587f91392933374d017bcbe1
                );
        else if (i == 29)
            return
                bytes32(
                    0x8869ff2c22b28cc10510d9853292803328be4fb0e80495e8bb8d271f5b889636
                );
        else if (i == 30)
            return
                bytes32(
                    0xb5fe28e79f1b850f8658246ce9b6a1e7b49fc06db7143e8fe0b4f2b0c5523a5c
                );
        else if (i == 31)
            return
                bytes32(
                    0x985e929f70af28d0bdd1a90a808f977f597c7c778c489e98d3bd8910d31ac0f7
                );
        else revert("Index out of bounds");
    }
}

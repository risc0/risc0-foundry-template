alloy::sol!(
    #[sol(rpc, all_derives)]
    interface ITornado {
        function deposit(bytes32 _commitment) external payable;

        function withdraw(
            bytes calldata _proof,
            bytes32 _root,
            bytes32 _nullifierHash
        ) external payable;
    }
);

alloy::sol!(
    event Deposit(
        bytes32 indexed commitment,
        uint32 leafIndex,
        uint256 timestamp
    );
);

alloy::sol!(
    #[sol(rpc, all_derives)]
    interface ITornado {
        function deposit(bytes32 _commitment) external payable;

        function withdraw(
            bytes calldata _proof,
            bytes32 _root,
            bytes32 _nullifierHash,
            address payable _recipient,
            address payable _relayer,
            uint256 _fee,
            uint256 _refund
        ) external payable;
    }
);

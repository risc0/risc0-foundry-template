set dotenv-filename := ".env.anvil"
set positional-arguments

start-devnet:
    anvil

deploy:
    forge script script/Deploy.s.sol:EthMixerDeploy --rpc-url $RPC_URL --private-key $ETH_WALLET_PRIVATE_KEY --broadcast

deposit:
    RUST_LOG_LEVEL=info cargo run --bin client -- deposit

@withdraw spending_key:
    cargo run --bin client -- withdraw $1

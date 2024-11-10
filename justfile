
start-devnet:
    anvil

deploy:
    forge script script/Deploy.s.sol:TornadoDeploy --rpc-url 127.0.0.1:8545 --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 --broadcast --verify -vvvv
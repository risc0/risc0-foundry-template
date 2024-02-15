# Apps


## Publisher
This template provides an application example, [publisher], that lets you send an off-chain proof request to the [Bonsai] proving service and publish the received proofs directly to your deployed app contract.

### Usage

```bash
Usage: publisher --chain-id <CHAIN_ID> --eth-wallet-private-key <ETH_WALLET_PRIVATE_KEY> --rpc-url <RPC_URL> --contract <CONTRACT> --input <INPUT>

Options:
      --chain-id <CHAIN_ID>
          Ethereum chain ID
      --eth-wallet-private-key <ETH_WALLET_PRIVATE_KEY>
          Ethereum Node endpoint [env: ETH_WALLET_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80]
      --rpc-url <RPC_URL>
          Ethereum Node endpoint
      --contract <CONTRACT>
          Application contract address on Ethereum
  -i, --input <INPUT>
          The hex-encoded input to provide to the guest binary
  -h, --help
          Print help
  -V, --version
          Print version
```

[publisher]: (./src/bin/publisher.rs)
[Bonsai]: https://dev.bonsai.xyz/
## Bonsai Deployment Guide
> **Note: This software is not production ready. Do not use in production.**

Welcome to the [Bonsai] Deployment guide! 

Once you've written your [contracts] and your [methods], and [tested] your program, you're ready to deploy your contract. You can either:
- Deploy your project to a local network
- Deploy to a testnet

## Deploy your project on a local network

You can deploy your contracts and run an end-to-end test or demo as follows:

1. Start a local testnet with `anvil` by running:

    ```bash
    anvil
    ```

    Once anvil is started, keep it running in the terminal, and switch to a new terminal.

2. Deploy an `IBonsaiRelay` contract by running:

    ```bash
    RISC0_DEV_MODE=true forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast
    ```

3. Check the logs for the address of the deployed `BonsaiTestRelay` contract and your application contract.
   Save them to a couple of environment variables to reference later.

    ```bash
    export BONSAI_RELAY_ADDRESS="#copy relay address from the deploy logs#"
    export APP_ADDRESS="#copy app address from the deploy logs#"
    ```

4. Start the Bonsai Ethereum Relay by running:

    ```bash
    RISC0_DEV_MODE=true cargo run --bin bonsai-ethereum-relay-cli -- run --relay-address "$BONSAI_RELAY_ADDRESS"
    ```

    The relay will keep monitoring the chain for callback requests, generated when your contract calls `bonsaiRelay.requestCallback(...)`, and relay their result back to your contract after computing them.
    Keep the relay running and switch to a new terminal.

    Setting `RISC0_DEV_MODE=true` deploys the `BonsaiTestRelay`, for use in local development and testing, instead of the fully verifying `BonsaiRelay` contract.
    See the section below on using the fully-verifying relay for more information on this setting and testnet deployment.

**Interact with your deployment:**

You now have a locally running testnet and relay deployment that you can interact with using `cast`, a wallet, or any application you write.

1. Send a transaction to the starter contract:

    ```bash
    cast send --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d --gas-limit 100000 "$APP_ADDRESS" 'calculateFibonacci(uint256)' 5
    ```

2. Check the relayed result:

    ```bash
    cast call "$APP_ADDRESS" 'fibonacci(uint256)' 5
    ```

**Deploy a new version of your application:**

When you want to deploy a new version of the application contract, run the following command with the relay contract address noted earlier.
Set `DEPLOY_UPLOAD_IMAGES=true` if you modified your guest and need to upload a new version to Bonsai.

```bash
RISC0_DEV_MODE=true DEPLOY_RELAY_ADDRESS="$APP_ADDRESS" DEPLOY_UPLOAD_IMAGES=true forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast
```

This will deploy only your application address and upload any updated images.
The existing relay contract and, by setting `DEPLOY_RELAY_ADDRESS`, the running relay will continue to be used.

**Use the fully verifying relay:**

In each of the commands above, the environment variable `RISC0_DEV_MODE=true` is added.
With this environment variable set, the `BonsaiTestRelay` contract is used, which does not check callbacks for authentication.
This provides fast development, allowing you to iterate on your application.

When it's time to deploy you application to a live chain, such as the Sepolia testnet, you should remove this environment or set `RISC0_DEV_MODE=false`.
When unset, or set to `false`, the fully-verifying `BonsaiRelay` contract will be used and all callbacks will require a [Groth16 SNARK proof] for authentication.
This is what provides the security guarantees of Bonsai, that only legitimate outputs from your guest program can be sent to your application contract.

Producing SNARK receipts that are verifiable on-chain requires the Bonsai proving service.
See the [Configuring Bonsai](#Configuring Bonsai) section below for more information about using the Bonsai proving service.

You can also deploy on a testnet by following the instructions described in [Deploy your project on a testnet](#deploy-your-project-on-a-testnet).
If you want to know more about the relay, you can follow this [link](https://github.com/risc0/risc0/tree/main/bonsai/ethereum-relay).

### Off-chain Callback Request

The Relay exposes an HTTP REST API interface that can be used to directly send *off-chain* callback requests to it, as an alternative to the on-chain requests.
It also provides an SDK in Rust that can be used to interact with it. You can check out this [example](relay/examples/callback_request.rs.rs).

Assuming that Anvil and the Relay are running and both an `IBonsaiRelay` and the `BonsaiStarter` app contract are deployed (first 4 steps of the previous section), you can send a callback request directly to the Relay by running:

```bash
cargo run --example callback_request "$APP_ADDRESS" 10
```

This example's arguments are the `BonsaiStarter` contract address and the number, N, to compute the Nth Fibonacci number.
You may need to change these values accordingly.

Just as with on-chain callback requests, you can check the relayed result

```bash
cast call "$APP_ADDRESS" 'fibonacci(uint256)' 10
```

The Relay source code with its SDK can be found in the [risc0/risc0] github repo.

## Deploy your project on a testnet

You can deploy your contracts on a testnet such as `Sepolia` and run an end-to-end test or demo as follows:

1. Get access to Bonsai and an Ethereum node running on a given testnet, e.g., Sepolia (in this example, we will be using [alchemy](https://www.alchemy.com/) as our Ethereum node provider) and export the following environment variables:

    ```bash
    export BONSAI_API_KEY="YOUR_API_KEY" # see form linked in the previous section
    export BONSAI_API_URL="BONSAI_URL" # provided with your api key
    export ALCHEMY_API_KEY="YOUR_ALCHEMY_API_KEY" # the API_KEY provided with an alchemy account
    export DEPLOYER_PRIVATE_KEY="YOUR_WALLET_PRIVATE_KEY" # the private key of your Ethereum testnet wallet e.g., Sepolia
    ```

2.  Deploy an `IBonsaiRelay` contract by running:

    ```bash
    RISC0_DEV_MODE=false forge script script/Deploy.s.sol --rpc-url https://eth-sepolia.g.alchemy.com/v2/$ALCHEMY_API_KEY --broadcast
    ```

3. Check the logs for the address of the deployed `BonsaiRelay` contract and your application contract.
   Save them to a couple of environment variables to reference later.

    ```bash
    export BONSAI_RELAY_ADDRESS="#copy relay address from the deploy logs#"
    export APP_ADDRESS="#copy app address from the deploy logs#"
    ```

4. Start the Bonsai Ethereum Relay by running:

    ```bash
    RISC0_DEV_MODE=false cargo run --bin bonsai-ethereum-relay-cli -- run --relay-address "$BONSAI_RELAY_ADDRESS" --eth-node wss://eth-sepolia.g.alchemy.com/v2/$ALCHEMY_API_KEY --eth-chain-id 11155111 --private-key "$DEPLOYER_PRIVATE_KEY"
    ```

    The relay will keep monitoring the chain for callback requests, generated when your contract calls `bonsaiRelay.requestCallback(...)`, and relay their result back to your contract after computing them.
    Keep the relay running and switch to a new terminal.

**Interact with your deployment:**

You now have a deployment on a testnet that you can interact with using `cast`, a wallet, or any application you write.

1. Send a transaction to the starter contract:

    ```bash
    cast send --rpc-url https://eth-sepolia.g.alchemy.com/v2/$ALCHEMY_API_KEY --private-key "$DEPLOYER_PRIVATE_KEY" --gas-limit 100000 "$APP_ADDRESS" 'calculateFibonacci(uint256)' 5
    ```

2. Check the relayed result:

    ```bash
    cast call --rpc-url https://eth-sepolia.g.alchemy.com/v2/$ALCHEMY_API_KEY "$APP_ADDRESS" 'fibonacci(uint256)' 5
    ```

[Bonsai]: https://risczero.com/bonsai
[contracts]: https://github.com/risc0/bonsai-foundry-template/tree/main/contracts
[methods]: https://github.com/risc0/bonsai-foundry-template/tree/main/methods
[tested]: https://github.com/risc0/bonsai-foundry-template/tree/main#test-your-project
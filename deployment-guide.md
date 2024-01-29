# Bonsai Deployment Guide
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

2. Set your private key:

    ```bash
    export ETH_WALLET_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
    ```

3. Deploy your contract by running:
    > This requires that `ETH_WALLET_PRIVATE_KEY` be set.

    ```bash
    forge script --rpc-url http://localhost:8545 --broadcast script/Deploy.s.sol
    ```

    This command should output something similar to:
    
    ```bash
    ...
    == Logs ==
    Deployed RiscZeroGroth16Verifier to 0x5FbDB2315678afecb367f032d93F642f64180aa3
    Deployed EvenNumber to 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
    ...
    ```

### Interact with your deployment:

1. Query the state:
    ```bash
    cast call --rpc-url http://localhost:8545 0xe7f1725e7734ce288f8367e1bb143e90bb3f0512 'get()(uint256)'
    ```

2. Publish a new state
    > ***Note:*** *This requires having access to a Bonsai API Key. To request an API key [complete the form here](https://bonsai.xyz/apply).*

    ```bash
    RISC0_DEV_MODE=false BONSAI_API_URL="BONSAI_API_URL" BONSAI_API_KEY="BONSAI_API_KEY" cargo run --release -- publish \
        --chain-id=31337 \
        --rpc-url=http://localhost:8545 \
        --contract=e7f1725E7734CE288F8367e1Bb143E90bb3F0512 \
        --input=12345678
    ```

## Deploy your project on a testnet

You can deploy your contracts on a testnet such as `Sepolia` and run an end-to-end test or demo as follows:

1. Get access to Bonsai and an Ethereum node running on a given testnet, e.g., Sepolia (in this example, we will be using [alchemy](https://www.alchemy.com/) as our Ethereum node provider) and export the following environment variables:

    ```bash
    export BONSAI_API_KEY="YOUR_API_KEY" # see form linked in the previous section
    export BONSAI_API_URL="BONSAI_API_URL" # provided with your api key
    export ALCHEMY_API_KEY="YOUR_ALCHEMY_API_KEY" # the API_KEY provided with an alchemy account
    export ETH_WALLET_PRIVATE_KEY="YOUR_WALLET_PRIVATE_KEY" # the private key of your Ethereum testnet wallet e.g., Sepolia
    ```

2.  Deploy your contract by running:

    ```bash
    forge script script/Deploy.s.sol --rpc-url https://eth-sepolia.g.alchemy.com/v2/$ALCHEMY_API_KEY --broadcast
    ```

     This command should output something similar to:
    
    ```bash
    ...
    == Logs ==
    Deployed RiscZeroGroth16Verifier to 0x5FbDB2315678afecb367f032d93F642f64180aa3
    Deployed EvenNumber to 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
    ...
    ```

    Save the `EvenNumber` contract address to an env variable:

    ```bash
    export EVEN_NUMBER_ADDRESS=0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
    ```


## Interact with your deployment:

1. Query the state:
    ```bash
    cast call --rpc-url https://eth-sepolia.g.alchemy.com/v2/$ALCHEMY_API_KEY $EVEN_NUMBER_ADDRESS 'get()(uint256)'
    ```

2. Publish a new state
    > ***Note:*** *This requires having access to a Bonsai API Key. To request an API key [complete the form here](https://bonsai.xyz/apply).*

    ```bash
    RISC0_DEV_MODE=false cargo run --release -- publish \
        --chain-id=11155111 \
        --rpc-url=https://eth-sepolia.g.alchemy.com/v2/$ALCHEMY_API_KEY \
        --contract=$EVEN_NUMBER_ADDRESS \
        --input=12345678
    ```

[Bonsai]: https://risczero.com/bonsai
[contracts]: https://github.com/risc0/bonsai-foundry-template/tree/main/contracts
[methods]: https://github.com/risc0/bonsai-foundry-template/tree/main/methods
[tested]: https://github.com/risc0/bonsai-foundry-template/tree/main#test-your-project
[Groth16 SNARK proof]: https://www.risczero.com/news/on-chain-verification

# Bonsai Foundry Template

Starter template for writing an application using [Bonsai].

This repository implements an application on Ethereum utilizing Bonsai as a coprocessor to the smart contract application.
It provides a starting point for building powerful new applications on Ethereum that offload computationally intensive
(or difficult to implement) tasks to be proven by the [RISC Zero] [zkVM], with verified results sent to your Ethereum contract.

https://github.com/risc0/bonsai-foundry-template/assets/15272444/5f2bc82d-f01a-437d-b515-c3747ffcedb9

## Dependencies

1. [Rust and Cargo](https://rustup.rs)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2. [Foundry](https://getfoundry.sh/)
```bash
curl -L https://foundry.paradigm.xyz | bash
```

***Hint:*** *If you're installing Foundry for the first time, be sure to quit your terminal before reopening it and running the `foundryup` command.*

## Project Setup
1. Use Foundry to create a new project from this template at a location of your choosing (this example command will create it at `./my-project`) 

```bash
forge init -t risc0/bonsai-foundry-template ./my-project
```

## Get Started

Get started writing your application by modifying these key files:

* Replace `contracts/BonsaiStarter.sol` with your on-chain application logic.
* Replace `methods/guest/src/bin/fibonacci.rs` with your [zkVM guest program].

Associated build configuration files and tests are discussed along with the [project structure](#project-structure) below.

## Test Your Project
With this Foundry template, you can write a zkVM Rust program in the `/methods/guest` directory which Solidity contracts in  `/contracts` can call into and in return receive a proof or 'receipt' of execution.
### Test your zkVM program
To check if your zkVM program will compile, and generate any errors if not, run the Rust compiler with

```bash
cargo build
```

If you've written tests in your zkVM progam, run them with
```bash
cargo test
```
***Hint:*** *To learn more about our RISC-V zkVM [visit the docs](https://dev.risczero.com/zkvm) or for a thorough walkthrough, follow the [Factors Tutorial here](https://github.com/risc0/risc0/tree/main/examples/factors#tutorial).*

### Test your solidity integration with the zkVM
To test both your Solidity contracts and their interaction with your zkVM program, run

```bash
forge test
```

***Hint:*** *To learn more about Foundry's `forge` command and the other helpful utilities Foundry provides, visit their docs: (https://book.getfoundry.sh/forge/tests)[https://book.getfoundry.sh/forge/tests].*

### Deploy your project on a local network
You can deploy your contracts and run an end-to-end test or demo as follows:

1. Start an anvil instance, if you want a local testnet, by running:
```
anvil
```
Once anvil is started, keep it running in the terminal, and switch to a new terminal.

2. Deploy the `BonsaiRelay` contract by running:
```
forge script scripts/Deploy.s.sol:Relay --rpc-url http://localhost:8545 --broadcast
```

3. Start the Bonsai Ethereum Relay by running:
```
RELAY_ADDRESS=0x5FbDB2315678afecb367f032d93F642f64180aa3 BONSAI_API_URL=http://localhost:8081 BONSAI_API_KEY=none cargo run --bin bonsai-ethereum-relay-cli -- run --eth-node ws://localhost:8545 --eth-chain-id 31337 --private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```
The relay will keep monitoring the chain for callback requests and relay their result back after computing them. You should keep this terminal instance running the relay in the foreground and switch to a new terminal.
If needed, you should modify the environment variables to reflect your setup. For instance, if you want to prove remotely via Bonsai, set `BONSAI_API_URL` and `BONSAI_API_KEY` accordingly.
Moreover, if you want to run the relay on a remote Ethereum network, you can use a different `--eth-node`, `--eth-chain-id` and `--private-key`.

4. On a new terminal, you can run the following forge script to deploy your `StarterContract`:
```
RELAY_ADDRESS=0x5FbDB2315678afecb367f032d93F642f64180aa3 BONSAI_API_URL=http://localhost:8081 BONSAI_API_KEY=none METHOD_NAME=FIBONACCI forge script scripts/Deploy.s.sol:Starter --rpc-url http://localhost:8545 --broadcast
```
Again, you can change the environment variables to reflect your setup.

**Now you can test your deployment as follows:**
1. Send a transaction to the starter contract:
```
cast send --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d --gas-limit 100000 0xe7f1725e7734ce288f8367e1bb143e90bb3f0512 'calculateFibonacci(uint256)' 5
```

2. Check the relayed result:
```
cast call 0xe7f1725e7734ce288f8367e1bb143e90bb3f0512 'fibonacci(uint256)' 5
```

## Proving Modes
The foundry template supports three different proving modes:
1. `local` - By default, only the [executor](https://www.dev.risczero.com/docs/terminology#executor) runs your zkVM program and no proof is generated. Because there is no proving, this will be the fastest way to test. 
2. `bonsai` - A proof of execution is generated by the Bonsai API.

https://github.com/risc0/bonsai-foundry-template/assets/15272444/db2b25c0-61b1-4884-9f8e-1d3641decf4a

Configure your preferred mode by setting the `BONSAI_PROVING` env variable.
```bash
export BONSAI_PROVING=local
```
After setting your preferred proving mode, use `forge test` to build and run your application. 
### Configuring Bonsai for Eth Waterloo
If you're at Eth Waterloo and want to run your proofs on Bonsai, complete [the form here](https://forms.gle/Ships93xqY4c8zA37) to request an API key. If you haven't received a key within a few hours drop a message in the [RISC Zero discord](https://discord.gg/risczero) **#eth-waterloo** channel.

To prove in `bonsai` mode, two additional environment variables are required
```bash
export BONSAI_API_KEY="YOUR_API_KEY" #see form linked above
export BONSAI_API_URL="BONSAI_URL" #provided with your api key
BONSAI_PROVING=bonsai forge test
```

## Project Structure

Below are the primary files in the project directory

```text
.
├── Cargo.toml                      // Definitions for cargo and rust
├── foundry.toml                    // Definitions for foundry
├── contracts                       // Your Ethereum contracts live here
│   ├── BonsaiStarter.sol           // Starter template for basic callback contract
│   └── BonsaiStarterLowLevel.sol   // Starter template for low-level callback contract
├── tests                           // Your Ethereum contract tests live here
│   ├── BonsaiStarter.t.sol         // Tests for basic callback contract
│   └── BonsaiStarterLowLevel.t.sol // Tests for low-level callback contract
└── methods                         // [zkVM guest programs] are built here
    ├── Cargo.toml
    ├── build.rs                    // Instructions for the risc0-build rust crate
    ├── guest                       // A rust crate containing your [zkVM guest programs]
    │   ├── Cargo.toml
    │   └── src
    │       └── bin                 // Your [zkVM guest programs] live here
    │           └── fibonacci.rs    // Example [guest program] for fibonacci number calculation
    └── src
        ├── main.rs                 // Glue binary for locally testing Bonsai applications
        └── lib.rs                  // Built RISC Zero guest programs are compiled into here
```

### Contracts

Ethereum contracts should be written in the `contracts` directory, where the two primary starter template contracts live.
The Solidity libraries for Bonsai can be found in `lib/bonsai-lib-sol/src`.

Contracts are built and tested with [forge], which is part of the [Foundry] toolkit.
Tests are defined in the `tests` directory.

### Methods

[RISC Zero] guest programs are defined in the `methods` directory.
This is where you will define one or more guest programs to act as a coprocessor to your on-chain logic.
More example of what you can do in the guest can be found in the [RISC Zero examples].

Code in the `methods/guest` directory will be compiled into one or more [RISC-V] binaries.
Each will have a corresponding image ID, which is a hash identifying the program.
When deploying your application, you will upload your binary to Bonsai where the guest will run when requested.
The image ID will be included in the deployment of the smart contracts to reference your guest program living in Bonsai.

Build configuration for the methods is included in `methods/build.rs`.

[Bonsai]: https://dev.bonsai.xyz/
[RISC Zero]: https://www.risczero.com/
[RISC Zero examples]: https://github.com/risc0/risc0/tree/main/examples
[RISC-V]: https://www.risczero.com/docs/reference-docs/about-risc-v
[Foundry]: https://getfoundry.sh/
[zkVM]: https://www.dev.risczero.com/terminology#zero-knowledge-virtual-machine-zkvm
[zkVM guest program]: https://www.dev.risczero.com/terminology#guest-program
[zkVM guest programs]: https://www.dev.risczero.com/terminology#guest-program
[guest program]: https://www.dev.risczero.com/terminology#guest-program
[proof]: https://www.dev.risczero.com/terminology#validity-proof

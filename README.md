# Bonsai Foundry Template

> **Note: This software is not production ready. Do not use in production.**

Starter template for writing an application using [RISC Zero] and Ethereum.

This repository implements an application on Ethereum utilizing [Bonsai] as a [coprocessor] to the smart contract application.
It provides a starting point for building powerful new applications on Ethereum that offload computationally intensive (i.e. gas expensive), or difficult to implement, tasks to be proven by the [RISC Zero zkVM].
Verifiable results are sent to your Ethereum contract.

[RISC Zero]: https://www.risczero.com/
[Bonsai]: https://dev.bonsai.xyz/
[coprocessor]: https://twitter.com/RiscZero/status/1677316664772132864
[RISC Zero zkVM]: https://dev.risczero.com/zkvm

## Overview

The picture below shows a simplified overview of how users can integrate [Bonsai] into their Ethereum smart contracts:

![Bonsai Foundry Template Diagram](images/bonsai-foundry-template.png)

1. Run your application logic in the [RISC Zero zkVM]. The provided Command Line Interface (CLI) sends an off-chain proof request to the Bonsai proving service.
2. Bonsai generates the computation result, written to the [journal], and a SNARK proof of its correctness.
3. The CLI submits this proof and journal on-chain to your app contract for validation.
4. Your app contract calls the [RISC Zero Verifier] to validate the proof. If the verification is successful, the journal is deemed trustworthy and can be safely used.

[journal]: https://dev.risczero.com/terminology#journal
[RISC Zero Verifier]: https://github.com/risc0/risc0/blob/release-0.20/bonsai/ethereum/contracts/IRiscZeroVerifier.sol

## Dependencies

First, [install Rust] and [Foundry], and then restart your terminal. Next, you will need to install the `cargo risczero` tool.

```sh
# Install Rust
curl https://sh.rustup.rs -sSf | sh
# Install Foundry
curl -L https://foundry.paradigm.xyz | bash
```

We'll use `cargo binstall` to get `cargo-risczero` installed. See [cargo-binstall] for more details.

```sh
cargo install cargo-binstall
cargo binstall cargo-risczero
```

Next we'll need to install the `risc0` toolchain with:

```sh
cargo risczero install
```

Now you have all the tools you need to develop and deploy an application with RISC Zero.

[install Rust]: https://doc.rust-lang.org/cargo/getting-started/installation.html
[Foundry]: https://getfoundry.sh/
[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall#cargo-binaryinstall

## Quick Start

First, install the RISC Zero toolchain using the [instructions above].

Now, you can initialize a new Bonsai project at a location of your choosing:

```sh
forge init -t risc0/bonsai-foundry-template ./my-project
```

Congratulations! You've just started your first RISC Zero project.

Your new project consists of:

- a [zkVM program] (written in Rust), which specifies a computation that will be proven;
- a [app contract] (written in Solidity), which receives the response;
- a [guest interface] (written in Rust), which lets you define how to parse and serialize the guest input and calldata so that the [RISC Zero zkVM] and Bonsai can interact with your contract.

[instructions above]: #dependencies
[zkVM program]: ./methods/guest/src/bin
[app contract]: ./contracts
[guest interface]: ./cli

### Run the Tests

- Use `cargo test` to run the tests in your zkVM program.
- Use `RISC0_DEV_MODE=true forge test -vvv` to test your Solidity contracts and their interaction with your zkVM program.

## Develop Your Application

To build your application, you'll need to make changes in three folders:

- write the code you want proven in the [methods] folder
- write the on-chain part of your project in the [contracts] folder
- write the guest interface in the [cli] folder

[methods]: ./methods
[cli]: ./cli
[contracts]: ./contracts

### Configuring Bonsai

***Note:*** *The Bonsai proving service is still in early Alpha. To request an API key [complete the form here](https://bonsai.xyz/apply).*

With the Bonsai proving service, you can produce a [Groth16 SNARK proof] that is verifiable on-chain.
You can get started by setting the following environment variables with your API key and associated URL.

```bash
export BONSAI_API_KEY="YOUR_API_KEY" # see form linked above
export BONSAI_API_URL="BONSAI_URL" # provided with your api key
```

<!-- TODO(victor): Rename the RiscZeroGroth16VerifierTest -->
Now if you run `forge test` with `RISC0_DEV_MODE=false`, the test will run as before, but will additionally use the fully verifying `RiscZeroGroth16Verifier` contract instead of `RiscZeroGroth16VerifierTest` and will request a SNARK receipt from Bonsai.

```bash
RISC0_DEV_MODE=false forge test -vvv
```

[Groth16 SNARK proof]: https://www.risczero.com/news/on-chain-verification

## Deploy Your Application

When you're ready, follow the [deployment guide] to get your application running on [Sepolia].

[deployment guide]: /deployment-guide.md
[Sepolia]: https://www.alchemy.com/overviews/sepolia-testnet

## Project Structure

Below are the primary files in the project directory

```text
.
├── Cargo.toml                      // Configuration for Cargo and Rust
├── foundry.toml                    // Configuration for Foundry
├── contracts
│   └── EvenNumber.sol              // Basic example contract for you to modify
├── tests
│   └── EvenNumber.t.sol            // Tests for the basic example contract
├── methods
│   ├── Cargo.toml
│   ├── guest
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── bin                 // You can add additionally guest prgrams to this folder
│   │           └── is_even.rs      // Example guest program for cheking if a number is even
│   └── src
│       └── lib.rs                  // Compiled image IDs and tests for your guest programs
└── cli
    ├── Cargo.toml
    └── src
        ├── interface.rs            // Interface for interacting with your contract
        └── main.rs                 // CLI for interacting with your application
```

# RISC Zero Foundry Template

> Prove computation with the [RISC Zero zkVM] and verify the results in your Ethereum contract.

This repository implements an example application on Ethereum utilizing RISC Zero as a [coprocessor] to the smart contract application.
It provides a starting point for building powerful new applications on Ethereum that offload work that is computationally intensive (i.e. gas expensive), or difficult to implement in Solidity (e.g. ed25519 signature verification, or HTML parsing).

<!-- TODO(#100) Integrate support for Steel more directly into this repo -->
Integrate with [Steel][steel-repo] to execute view calls and simulate transactions on Ethereum. Check out the [ERC-20 counter][erc20-counter] demo to see an example.

## Overview

Here is a simplified overview of how devs can integrate RISC Zero, with [Bonsai] proving, into their Ethereum smart contracts:

![RISC Zero Foundry Template Diagram](images/risc0-foundry-template.png)

1. Run your application logic in the [RISC Zero zkVM]. The provided [publisher] app sends an off-chain proof request to the [Bonsai] proving service.
2. [Bonsai] generates the program result, written to the [journal], and a SNARK proof of its correctness.
3. The [publisher] app submits this proof and journal on-chain to your app contract for validation.
4. Your app contract calls the [RISC Zero Verifier] to validate the proof. If the verification is successful, the journal is deemed trustworthy and can be safely used.

## Dependencies

First, [install Rust] and [Foundry], and then restart your terminal.

```sh
# Install Rust
curl https://sh.rustup.rs -sSf | sh
# Install Foundry
curl -L https://foundry.paradigm.xyz | bash
```

Next, you will need to install the `cargo risczero` tool.
We'll use [`cargo binstall`][cargo-binstall] to get `cargo-risczero` installed, and then install the `risc0` toolchain.
See [RISC Zero installation] for more details.

```sh
cargo install cargo-binstall
cargo binstall cargo-risczero
cargo risczero install
```

Now you have all the tools you need to develop and deploy an application with [RISC Zero].

## Quick Start

First, install the RISC Zero toolchain using the [instructions above](#dependencies).

Now, you can initialize a new RISC Zero project at a location of your choosing:

```sh
forge init -t risc0/bonsai-foundry-template ./my-project
```

Congratulations! You've just started your first RISC Zero project.

Your new project consists of:

- a [zkVM program] (written in Rust), which specifies a computation that will be proven;
- a [app contract] (written in Solidity), which uses the proven results;
- a [publisher] which makes proving requests to [Bonsai] and posts the proof to Ethereum.
  We provide an example implementation, but your dApp interface or application servers could act as the publisher.

### Build the Code

- Builds for zkVM program, the publisher app, and any other Rust code.

  ```sh
  cargo build
  ```

- Build your Solidity smart contracts

  > NOTE: `cargo build` needs to run first to generate the `ImageID.sol` contract.

  ```sh
  forge build
  ```

### Run the Tests

- Tests your zkVM program.

  ```sh
  cargo test
  ```

- Test your Solidity contracts, integrated with your zkVM program.

  ```sh
  RISC0_DEV_MODE=true forge test -vvv 
  ```

- Run the same tests, with the full zkVM prover rather than dev-mode, by setting `RISC0_DEV_MODE=false`.

  ```sh
  RISC0_DEV_MODE=false forge test -vvv
  ```

  Producing the [Groth16 SNARK proofs][Groth16] for this test requires running on an x86 machine with [Docker] installed, or using [Bonsai](#configuring-bonsai). Apple silicon is currently unsupported for local proving, you can find out more info in the relevant issues [here](https://github.com/risc0/risc0/issues/1520) and [here](https://github.com/risc0/risc0/issues/1749). 

## Develop Your Application

To build your application using the RISC Zero Foundry Template, you’ll need to make changes in three main areas:

- ***Guest Code***: Write the code you want proven in the [methods/guest](./methods/guest/) folder. This code runs off-chain within the RISC Zero zkVM and performs the actual computations. For example, the provided template includes a computation to check if a given number is even and generate a proof of this computation.
- ***Smart Contracts***: Write the on-chain part of your project in the [contracts](./contracts/) folder. The smart contract verifies zkVM proofs and updates the blockchain state based on the results of off-chain computations. For instance, in the [EvenNumber](./contracts/EvenNumber.sol) example, the smart contract verifies a proof that a number is even and stores that number on-chain if the proof is valid.
- ***Publisher Application***: Adjust the publisher example in the [apps](./apps/) folder. The publisher application bridges off-chain computation with on-chain verification by submitting proof requests, receiving proofs, and publishing them to the smart contract on Ethereum.

### Configuring Bonsai

***Note:*** *To request an API key [complete the form here](https://bonsai.xyz/apply).*

With the Bonsai proving service, you can produce a [Groth16 SNARK proof][Groth16] that is verifiable on-chain.
You can get started by setting the following environment variables with your API key and associated URL.

```bash
export BONSAI_API_KEY="YOUR_API_KEY" # see form linked above
export BONSAI_API_URL="BONSAI_URL" # provided with your api key
```

Now if you run `forge test` with `RISC0_DEV_MODE=false`, the test will run as before, but will additionally use the fully verifying `RiscZeroGroth16Verifier` contract instead of `MockRiscZeroVerifier` and will request a SNARK receipt from Bonsai.

```bash
RISC0_DEV_MODE=false forge test -vvv
```

### Deterministic Builds

By setting the environment variable `RISC0_USE_DOCKER` a containerized build process via Docker will ensure that all builds of your guest code, regardless of the machine or local environment, will produce the same [image ID][image-id].
The [image ID][image-id], and its importance to security, is explained in more detail in our [developer FAQ].

```bash
RISC0_USE_DOCKER=1 cargo build
```

> ***Note:*** *This requires having Docker installed and in your PATH. To install Docker see [Get Docker][Docker].*

## Deploy Your Application

When you're ready, follow the [deployment guide] to get your application running on [Sepolia] or Ethereum Mainnet.

## Project Structure

Below are the primary files in the project directory

```text
.
├── Cargo.toml                      // Configuration for Cargo and Rust
├── foundry.toml                    // Configuration for Foundry
├── apps
│   ├── Cargo.toml
│   └── src
│       └── lib.rs                  // Utility functions
│       └── bin                     
│           └── publisher.rs        // Example app to publish program results into your app contract 
├── contracts
│   ├── EvenNumber.sol              // Basic example contract for you to modify
│   └── ImageID.sol                 // Generated contract with the image ID for your zkVM program
├── methods
│   ├── Cargo.toml
│   ├── guest
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── bin                 // You can add additional guest programs to this folder
│   │           └── is_even.rs      // Example guest program for checking if a number is even
│   └── src
│       └── lib.rs                  // Compiled image IDs and tests for your guest programs
└── tests
    ├── EvenNumber.t.sol            // Tests for the basic example contract
    └── Elf.sol                     // Generated contract with paths the guest program ELF files.
```

[Bonsai]: https://dev.bonsai.xyz/
[Foundry]: https://getfoundry.sh/
[Docker]: https://docs.docker.com/get-docker/
[Groth16]: https://www.risczero.com/news/on-chain-verification
[RISC Zero Verifier]: https://github.com/risc0/risc0/blob/release-0.21/bonsai/ethereum/contracts/IRiscZeroVerifier.sol
[RISC Zero installation]: https://dev.risczero.com/api/zkvm/install
[RISC Zero zkVM]: https://dev.risczero.com/zkvm
[RISC Zero]: https://www.risczero.com/
[Sepolia]: https://www.alchemy.com/overviews/sepolia-testnet
[app contract]: ./contracts/
[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall#cargo-binaryinstall
[coprocessor]: https://www.risczero.com/news/a-guide-to-zk-coprocessors-for-scalability
[deployment guide]: /deployment-guide.md
[developer FAQ]: https://dev.risczero.com/faq#zkvm-application-design
[image-id]: https://dev.risczero.com/terminology#image-id
[install Rust]: https://doc.rust-lang.org/cargo/getting-started/installation.html
[journal]: https://dev.risczero.com/terminology#journal
[publisher]: ./apps/README.md
[zkVM program]: ./methods/guest/
[steel-repo]: https://github.com/risc0/risc0-ethereum/tree/main/steel
[erc20-counter]: https://github.com/risc0/risc0-ethereum/tree/main/examples/erc20-counter

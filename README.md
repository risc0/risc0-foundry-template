# Bonsai Foundry Template

Starter template for writing an application using [Bonsai].

This repository implements an application on Ethereum utilizing Bonsai as a coprocessor to the smart contract application.
It provides a starting point for building powerful new applications on Ethereum that offload computationally intensive,
or difficult to implement tasks to a [RISC Zero] guest, with verified results sent to your Ethereum contract.

## Getting Started

Start building your application by forking this template.

### Dependencies

* Rust and Cargo: https://rustup.rs
* Foundry: https://getfoundry.sh/

### Write Your App

Get started writing your application by modifying these key files:
* Replace `contracts/BonsaiStarter.sol` with your on-chain application logic.
* Replace `methods/guest/src/bin/fibonacci.rs` with your Bonsai coprocessor logic.

Associated build configuration files and tests are discussed along with the [project structure](#project-structure) below.

### Build

Running the following will build the RISC Zero guest program.

```bash
cargo build
```

While the following will build your Ethereum contracts.

```bash
forge build
```

### Test

Running the following will run the RISC Zero guest program tests.

```bash
cargo test
```

Running the following will run the Ethereum contract tests using your RISC Zero guest program, but without running
the expensive computations required to prove its behavior in zero-knowledge.

```bash
forge test
```

For testing with proof generation, which might take some time to complete, execute the following command instead:
```bash
PROVE_LOCALLY=1 forge test
```

For offloading your proof requests to a local Bonsai instance, you can execute the tests as follows:
```bash
BONSAI_ENDPOINT=http://localhost:8080 API_KEY=test_key forge test
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
└── methods                         // RISC Zero guest programs are built here
    ├── Cargo.toml                  
    ├── build.rs                    // Instructions for the risc0-build rust crate
    ├── guest                       // A rust crate containing your RISC Zero guest programs
    │   ├── Cargo.toml              
    │   └── src                     
    │       └── bin                 // Your RISC Zero guest programs live here
    │           └── fibonacci.rs    // Example program for fibonacci number calculation
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
[ethers]: https://docs.rs/ethers/latest/ethers/
[Cargo]: https://doc.rust-lang.org/cargo/
[RISC Zero examples]: https://github.com/risc0/risc0/tree/main/examples
[RISC-V]: https://www.risczero.com/docs/reference-docs/about-risc-v
[waitlist]: https://fmree464va4.typeform.com/to/t6hZD54Z
[Foundry]: https://getfoundry.sh/
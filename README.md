# Bonsai Foundry Template

> **Note: This software is not production ready. Do not use in production.**

Starter template for writing an application using [Bonsai].

This repository implements an application on Ethereum utilizing Bonsai as a [coprocessor] to the smart contract application.
It provides a starting point for building powerful new applications on Ethereum that offload computationally intensive, or difficult to implement, tasks to be proven by the [RISC Zero] [zkVM], with verifiable results sent to your Ethereum contract.

*For a 60 second overview of how this template and off-chain computation with Bonsai work, [check out the video here](https://www.youtube.com/watch?v=WDS8X8H9mIk).*

## Dependencies
First, [install Rust] and [Foundry], and then restart your terminal. Next, you will need to install the `cargo risczero tool`:

```bash
cargo install cargo-risczero
```

For the above commands to build successfully you will need to have installed the required dependencies. 

```bash
sudo apt install curl build-essential libssl-dev pkgconf
```

Next we'll need to install the `risc0` toolchain with:

```bash
cargo risczero install
```

## Quick Start
First, install the RISC Zero toolchain using the instructions above. 

Now, you can initialize a new Bonsai project at a location of your choosing: 

```bash
forge init -t risc0/bonsai-foundry-template ./my-project
```
Congratulations! You've just built your first Bonsai project.
Your new project consists of:
- a [`zkVM program`] (written in Rust), which specifies a computation that will be proven
- a [`contract`] (written in Solidity), which requests a proof and receives the response

[install Rust]: https://doc.rust-lang.org/cargo/getting-started/installation.html
[Foundry]: https://getfoundry.sh/
[`zkVM program`]: https://github.com/risc0/bonsai-foundry-template/tree/main/methods/guest/src/bin
[`contract`]: https://github.com/risc0/bonsai-foundry-template/tree/main/contracts

### Test Your Project
- Use `cargo build` to test compilation of your zkVM program.
- Use `cargo test` to run the tests in your zkVM program. 
- Use `forge test` to test your Solidity contracts and their interaction with your zkVM program.

### Configuring Bonsai
***Note:*** *The Bonsai proving service is still in early Alpha. To request an API key [complete the form here](https://bonsai.xyz/apply).*

With the Bonsai proving service, you can produce a [Groth16 SNARK proof] that is verifiable on-chain.
You can get started by setting the following environment variables with your API key and associated URL.

```bash
export BONSAI_API_KEY="YOUR_API_KEY" # see form linked above
export BONSAI_API_URL="BONSAI_URL" # provided with your api key
```

Now if you run `forge test` with `RISC0_DEV_MODE=false`, the test will run as before, but will additionally use the fully verifying `BonsaiRelay` contract instead of `BonsaiTestRelay` and will request a SNARK receipt from Bonsai.

```bash
RISC0_DEV_MODE=false forge test
```

## Next Steps
To build your application, you'll need to make changes in two folders: 
- write the code you want proven in the [methods] folder
- write the on-chain part of your project in the [contracts] folder

Then, you're ready to [deploy your project]. <br/>


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


[methods]: /methods
[contracts]: /contracts
[deploy your project]: /deployment-guide.md
[coprocessor]: https://twitter.com/RiscZero/status/1677316664772132864
[Bonsai]: https://dev.bonsai.xyz/
[Foundry]: https://getfoundry.sh/
[Groth16 SNARK proof]: https://www.risczero.com/news/on-chain-verification
[RISC Zero examples]: https://github.com/risc0/risc0/tree/main/examples
[RISC Zero]: https://www.risczero.com/
[RISC-V]: https://www.risczero.com/docs/reference-docs/about-risc-v
[https://book.getfoundry.sh/forge/tests]: https://book.getfoundry.sh/forge/tests
[receipt]: https://dev.risczero.com/zkvm/developer-guide/receipts
[risc0/risc0]: https://github.com/risc0/risc0/tree/main/bonsai/ethereum-relay
[zkVM guest program]: https://www.dev.risczero.com/terminology#guest-program
[zkVM]: https://www.dev.risczero.com/terminology#zero-knowledge-virtual-machine-zkvm

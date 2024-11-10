# Coin Mixer with RISC Zero

A tornado-cash style coin mixer implemented using RISC Zero

## About

Implements a protocol very similar to the original Tornado-cash with the following changes:

- Uses sha256 hashing for the nullifier and commitment tree instead of Pederson and MiMC hashes
  - Originally Pederson and MiMC were chosen as they are cheap to compute in Groth16
  - This uses sha256 making the protocol simpler and cheaper to implement on-chain
- Removes the withdrawal fee functionality. Mostly to keep the example simple.

The protocol works as follows:

### Deposit

The depositor generates a nullifier, $k$, and secret value, $r$, locally. These are hashed together to produce a note commitment

$C = H(k||r)$.

This is submitted to the contract along with a pre-determined amount of eth (this example uses 1 Eth sized notes). The tuple ($k$, $r$) makes up the spending key for this note.

Upon receiving the eth the contract appends the note commitment to its internal incremental Merkle tree and stores the tree root, $R$. It also saves the commitment to ensure it cannot be used again.

### Withdrawal

To spend the withdrawer needs to construct a proof with the following form:

> I know $k$, $r$, $l$, $O(l)$
> such that
>
> - $h = H(k)
> - O(l) is a valid merkle proof for leaf $C = H(k||r)$ rooted at $R$

where $l$ is the leaf position of the note commitment they are attempting to spend, and $h$ is the nullifier hash. In this case $h$ and $R$ are the public inputs and $k$, $r$, $l$, and $O(l)$ are private inputs. The proof also needs to commit to the receiving address $A$ so that the withdrawal transaction is non-malleable.

To construct a valid merkle proof the withdrawer has to reconstruct the contract merkle tree locally. It does this by querying an RPC node for all deposit events and builds a tree locally with extracted the note commitments.

The contract verifies this proof, checks that the nullifier hash, tree root and receiving address match the journal, and checks this nullifier hash has not been used already. If satisfied it allows the withdrawer to transfer 1 note worth of Eth out of the contract to the receiver. It then stores the nullifier hash in contract state so it cannot be used again.

## Dependencies

First, [install Rust] and [Foundry], and then restart your terminal.

```sh
# Install Rust
curl https://sh.rustup.rs -sSf | sh
# Install Foundry
curl -L https://foundry.paradigm.xyz | bash
```

To install `rzup`, run the following command and follow the instructions:

```sh
curl -L https://risczero.com/install | bash
rzup
```

This repo uses the [just](https://github.com/casey/just) command runner. Install it with:

```sh
cargo install just
```

### Build the Code

- Update git submodules.

  ```sh
  git submodule update --init
  ```

- Builds for zkVM program, the publisher app, and any other Rust code.

  ```sh
  cargo build
  ```

- Build your Solidity smart contracts.

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

### Configuring Bonsai

**_Note:_** _To request an API key [complete the form here](https://bonsai.xyz/apply)._

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

> **_Note:_** _This requires having Docker installed and in your PATH. To install Docker see [Get Docker][Docker]._

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

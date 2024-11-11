# Simple Mixer

A tornado-cash style coin mixer implemented using RISC Zero

## About

Implements a protocol very similar to the original Tornado-cash with the following changes:

- Uses sha256 hashing for the nullifier and commitment tree instead of Pederson and MiMC hashes
  - Sha256 is cheaper in RISC Zero and cheaper on-chain
- Removes the withdrawal fee functionality. Mostly to keep the demo simple.

Why rewrite tornado cash with RISC Zero? Aside from being a nice example it opens up the possibility to compose additional proofs with the withdrawal proofs. For example it would be straightforward to add compliance checking to ensure that the withdrawer is a member of a whitelisted set without linking this identity to their account.

The protocol works as follows:

### Deposit

The depositor generates a nullifier, $k$, and secret value, $r$, locally. These are hashed together to produce a note commitment

$C = H(k||r)$.

This is submitted to the contract along with a pre-determined amount of eth (this example uses 1 Eth sized notes). The tuple ($k$, $r$) makes up the spending key for this note.

Upon receiving the deposit the contract appends the note commitment to its internal incremental Merkle tree and stores the tree root, $R$. It also saves the commitment to ensure it cannot be used again.

### Withdrawal

To spend the withdrawer needs to construct a proof with the following form:

> I know $k$, $r$, $l$, $O(l)$
> such that:
>
> - $h = H(k)
> - O(l) is a valid merkle proof for leaf $C = H(k||r)$ rooted at $R$

where $l$ is the leaf position of the note commitment they are attempting to spend, and $h$ is the nullifier hash. In this case $h$ and $R$ are the public inputs and $k$, $r$, $l$, and $O(l)$ are private inputs. The proof also needs to commit to the receiving address $A$ so that the withdrawal transaction is non-malleable.

To construct a valid merkle proof the withdrawer has to reconstruct the contract merkle tree locally. It does this by querying an RPC node for all deposit events and builds a tree locally with extracted the note commitments.

The contract verifies this proof, checks that the nullifier hash, tree root and receiving address match the journal, and checks this nullifier hash has not been used already. If satisfied it allows the withdrawer to transfer 1 note worth of Eth out of the contract to the receiver. It then stores the nullifier hash in contract state so it cannot be used again.

## Repo Contents

> [!NOTE]
> The contracts `Mixer.sol`, `EthMixer.sol` and `MerkleTreeWithHistory.sol` are modified versions of the [original Tornado cash contracts](https://github.com/tornadocash/tornado-core/tree/master/contracts) and should not be considered original code for this submission.

```
.
├── apps
│   ├── Cargo.toml
│   └── src
│       └── precompute_zero_nodes.rs // Utility to populate the precomputed zero notes in the merkle tree contract
│       └── bin
│           └── client/             // App to interact with the mixer. This performs the secret generation and proving and interacts with the contracts
├── core                            // Crate with common functionality between the client and the guest program
│   ├── Cargo.toml
│   └── src
│       └── lib.rs                  // exports the `ProofInput` type and encoding helpers
├── contracts
│   ├── Mixer.sol                   // Mixer implementation, this checks the proofs and stores the merkle tree and nullifiers
│   ├── EthMixer.sol                // Mixer impl specific to using Eth (rather than an erc20 token)
│   ├── MerkleTreeWithHistory.sol   // Incremental merkle tree implementation. Modified to use sha2
│   └── ImageID.sol                 // Generated contract with the image ID for your zkVM program
├── methods
│   ├── Cargo.toml
│   ├── guest
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── bin
│   │           └── can_spend.rs      // Guest program for doing a note spend check
│   └── src
│       └── lib.rs                  // Compiled image IDs and tests for the guest program
└── tests
    └── MerkleTree.t.sol            // Tests ensuring compatibility between on-chain and off-chain merkle tree
```

## Running the Demo!

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

- Builds for zkVM program and the client app

  ```sh
  cargo build
  ```

- Build your Solidity smart contracts.

  > NOTE: `cargo build` needs to run first to generate the `ImageID.sol` contract.

  ```sh
  forge build
  ```

### Run Locally

The easiest way to demo the mixer is using a local anvil devnet. Start an anvil instance and keep it running:

```sh
anvil
```

---

In another shell deploy the mixer contract with:

```sh
just deploy
```

and get the contract address from the deploy output

```
== Logs ==
  You are deploying on ChainID 31337
  Deployed RiscZeroGroth16Verifier to 0x9A676e781A523b5d0C0e43731313A708CB607508
  Deployed EthMixer to 0x0B306BF915C4d645ff596e518fAf3F9669b97016 <-----------THIS ONE
```

Set the contract and Bonsai API (optional, required for non-x86 arch) values in the [.env.anvil](./.env.anvil) file

```bash
export CONTRACT=""
...
export BONSAI_API_KEY="YOUR_API_KEY"
```

#### Depositing Eth to the mixer

Once the above is set up you can deposit 1 eth from the anvil test account by running:

```sh
just deposit
```

This will perform the secret generation and send a commitment along with 1 Eth to the mixer. The spending key hex will be written to std-out. Copy this for the next step.

#### Withdrawing from the mixer

To withdraw run with the spending key from above

```sh
just withdraw <spending-key-hex>
```

This needs to generate a spending proof so may take a minute or so. Once it has generated a proof it will submit it on-chain to be verified and if successful will trigger the withdrawal 1 note worth of Eth. Attempting to withdraw with the same spending key more than once will fail.

## Run the Tests

- Tests the zkVM program and client

  ```sh
  cargo test
  ```

- Test the Solidity contracts

  ```sh
  forge test -vvv
  ```

## References

- [Tornado cash paper](https://berkeley-defi.github.io/assets/material/Tornado%20Cash%20Whitepaper.pdf)
- [Tornado cash reference implementation](https://github.com/tornadocash/tornado-core)

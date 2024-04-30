# Walkthrough of the RISC Zero Foundry Template.

This walkthrough covers the Even Number example provided with the RISC Zero Foundry Template.

It is built with [Foundry](https://github.com/foundry-rs/foundry), a powerful set of tools for Ethereum smart contract development, including testing and deployment. The template leverages the RISC Zero zkVM to ensure the integrity of computations on the Ethereum blockchain. It includes a smart contract, guest code for off-chain computations, a publisher (CLI) application for interacting with Ethereum, and tests to ensure everything works as expected.

## Guest code overview

The guest code runs off-chain within the RISC Zero zkVM. It performs the actual computation to check if a given number is even, generating a proof of this computation. This code forms the basis for the off-chain computation part of the Ethereum ZK-Coprocessor.

### Guest code walkthrough:

```rust
use std::io::Read;
use alloy_primitives::U256;
use alloy_sol_types::SolValue;
use risc0_zkvm::guest::env;

fn main() {
    // Read the input data for this application.
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();
    // Decode and parse the input
    let number = <U256>::abi_decode(&input_bytes, true).unwrap();

    // Run the computation.
    // In this case, asserting that the provided number is even.
    assert!(!number.bit(0), "number is not even");

    // Commit the journal that will be received by the application contract.
    // Journal is encoded using Solidity ABI for easy decoding in the app contract.
    env::commit_slice(number.abi_encode().as_slice());
}
```

- **Input reading**: The code starts by reading the input data, which in this case is the number whose evenness is to be verified. This number is passed into the zkVM as a byte array.
- **Decoding the input**: The input bytes are decoded into an U256 type, which stands for an unsigned 256-bit integer. This is a common data type for representing numbers in Ethereum smart contracts due to its ability to handle large integers used in cryptographic operations.
- **Verification logic**: The core of the guest code is a simple assertion that checks if the least significant bit (bit(0)) of the number is 0 (indicating the number is even). If the number is not even, the assertion fails, and the zkVM will not produce a valid proof.
- **Committing the Journal**: If the assertion passes, the code commits to the journal (i.e., the output of the computation). The journal contains the encoded version of the number, prepared in such a way that the smart contract can easily decode it using Solidity's ABI (Application Binary Interface) decoding functions.

## Smart Contract overview

RISC Zero enables developers to write off-chain logic as guest code, and verify zkVM proofs (specifically a Groth16 SNARK) in their smart contract logic on chain. Guest logic, which checks if a number is even, is run off-chain and enforced on-chain. This model extends to any program, however complex, which can be written as a guest program of the RISC Zero zkVM via Rust.

The EvenNumber smart contract verifies a RISC Zero zkVM proof that a number is even.
In this simple example, if a proof of an even number is verified, we will set that even number in the smart contract's state.

### Key Components:
- **RISC Zero Verifier**: An interface to the RISC Zero verifier contract, which checks the validity of a zero-knowledge proof.
- **Image ID**: Identifier for a specific zkVM guest binary (ELF) that the contract accepts proofs from. The image ID is similar to the address of a smart contract. It uniquely represents the logic of that guest program, ensuring that only proofs generated from a pre-defined guest program (in this case, checking if a number is even) are considered valid.

### Contract walkthrough:

```solidity
pragma solidity ^0.8.20;

import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {ImageID} from "./ImageID.sol";

contract EvenNumber {
    IRiscZeroVerifier public immutable verifier;
    bytes32 public constant imageId = ImageID.IS_EVEN_ID;
    uint256 public number;

    constructor(IRiscZeroVerifier _verifier) {
        verifier = _verifier;
        number = 0;
    }

    function set(uint256 x, bytes32 postStateDigest, bytes calldata seal) public {
        bytes memory journal = abi.encode(x);
        require(verifier.verify(seal, imageId, postStateDigest, sha256(journal)));
        number = x;
    }

    function get() public view returns (uint256) {
        return number;
    }
}
```
#### Constructor

The constructor requires the address of the RISC Zero verifier contract as an argument. This address is then set to an immutable variable, ensuring that the verification process always refers to the specified verifier. Finally, initializes the number state variable to 0, representing the initial state of the stored number.

#### Setting the Even Number

The **set** function updates the stored number, but only if the provided SNARK proof validates that the number is indeed even.

##### Inputs:
- **x**: The new even number to store.
- **postStateDigest**: A hash representing the expected state of the zkVM after executing the computation.
- **seal**: The SNARK proof generated off-chain, proving **x** is even.

The function constructs a journal expected to match the zkVM's computation, encodes the new number **x**, and then calls the verifier's verify method with the proof (**seal**), **imageId**, **postStateDigest**, and the **journal**.
If verification succeeds (i.e., the proof is valid, and the journal matches), the contract updates the number state variable to **x**.

#### Getting the Stored Number

The **get** function allows anyone to retrieve the current even number stored in the contract. It simply returns the value of the number state variable.
The number will always be even by the guarantees of the zkVM.

## Publisher Application overview

The Publisher application is a critical component in the workflow that bridges the off-chain computation with the on-chain verification in the Ethereum blockchain. This application demonstrates how to send an off-chain proof request to Bonsai, the RISC Zero proving service, receive the proofs, and then publish these proofs directly to a deployed smart contract (**EvenNumber** contract) on Ethereum. It serves as the user interface for interacting with both the RISC Zero zkVM and the Ethereum blockchain.


### Key components
- **IEvenNumber Interface**: Generated automatically via the **sol!** macro, this interface mirrors the functions available in the EvenNumber smart contract, allowing the publisher application to construct calls to the smart contract's functions correctly.
- **BonsaiProver**: This component interacts with the Bonsai proving service to submit off-chain computations and receive back a proof (in this context, the proof that a number is even), a journal, and a post-state digest.
- **TxSender**: This component is responsible for creating and sending transactions to the Ethereum blockchain, particularly for calling functions on the deployed smart contract.


### Publisher Application walkthrough:

```rust
use alloy_primitives::U256;
use alloy_sol_types::{sol, SolInterface, SolValue};
use anyhow::{Result};
use apps::{BonsaiProver, TxSender};
use clap::Parser;
use methods::IS_EVEN_ELF;

sol! {
    interface IEvenNumber {
        function set(uint256 x, bytes32 post_state_digest, bytes calldata seal);
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(long)]
    chain_id: u64,
    #[clap(long, env)]
    eth_wallet_private_key: String,
    #[clap(long)]
    rpc_url: String,
    #[clap(long)]
    contract: String,
    #[clap(short, long)]
    input: U256,
}

fn main() -> Result<()> {
    // Parse CLI Arguments: The application starts by parsing command-line arguments provided by the user.
    let args = Args::parse();
    
    // Create a new transaction sender using the parsed arguments.
    let tx_sender = TxSender::new(args.chain_id, &args.rpc_url, &args.eth_wallet_private_key, &args.contract)?;

    // ABI encode input: Before sending the proof request to the Bonsai proving service, 
    // the input number is ABI-encoded to match the format expected by the guest code running in the zkVM.
    let input = args.input.abi_encode();

    // Request proof from the Bonsai proving service: The application sends the encoded input to the 
    // Bonsai proving service, requesting it to execute the is_even computation off-chain and generate the necessary proof.
    let (journal, post_state_digest, seal) = BonsaiProver::prove(IS_EVEN_ELF, &input)?;

    // Decode Journal: Upon receiving the proof, the application decodes the journal to extract 
    // the verified number. This ensures that the number being submitted to the blockchain matches 
    // the number that was verified off-chain.
    let x = U256::abi_decode(&journal, true)?;

    // Construct function call: Using the IEvenNumber interface, the application constructs 
    // the ABI-encoded function call for the set function of the EvenNumber contract. 
    // This call includes the verified number, the post-state digest, and the seal (proof).
    let calldata = IEvenNumber::IEvenNumberCalls::set(IEvenNumber::setCall { x, post_state_digest, seal }).abi_encode();

    // Initialize the async runtime environment to handle the transaction sending.
    let runtime = tokio::runtime::Runtime::new()?;

    // Send transaction: Finally, the TxSender component sends the transaction to the Ethereum blockchain, 
    // effectively calling the set function of the EvenNumber contract with the verified number and proof.
    runtime.block_on(tx_sender.send(calldata))?;

    Ok(())
}
```


## Testing overview

To complete the walkthrough, we're going to dive into the testing phase using a test script for the *EvenNumber* smart contract. Testing is crucial in smart contract development to ensure the code behaves as expected under various conditions. This particular test script is designed to validate the functionality of the *EvenNumber* contract, specifically its ability to accept and store only even numbers verified by a RISC Zero verifier.

### Key Components
- **setUp**: Initializes the test environment. It deploys a  RISC Zero verifier (or a mock) and the EvenNumber contract, setting the initial number to 0 and validating this setup.
- **test_SetEven**: Tests the set function of the EvenNumber contract with an even number. It runs (or simulates) the off-chain proof generation process for an even number, then calls the set function with this data, and finally checks if the contract's stored number matches the input.
- **test_SetZero**: Similar to *test_SetEven*, but specifically tests the edge case of setting the stored number to 0, which is also even. This ensures that boundary conditions are correctly handled by the contract.

### Test walkthrough
```solidity
pragma solidity ^0.8.20;

import {RiscZeroCheats} from "risc0/RiscZeroCheats.sol";
import {console2} from "forge-std/console2.sol";
import {Test} from "forge-std/Test.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {EvenNumber} from "../contracts/EvenNumber.sol";
import {Elf} from "./Elf.sol";

contract EvenNumberTest is RiscZeroCheats, Test {
    EvenNumber public evenNumber;

    function setUp() public {
        IRiscZeroVerifier verifier = deployRiscZeroVerifier();
        evenNumber = new EvenNumber(verifier);
        assertEq(evenNumber.get(), 0);
    }

    function test_SetEven() public {
        uint256 number = 12345678;
        (bytes memory journal, bytes32 post_state_digest, bytes memory seal) = prove(Elf.IS_EVEN_PATH, abi.encode(number));
        evenNumber.set(abi.decode(journal, (uint256)), post_state_digest, seal);
        assertEq(evenNumber.get(), number);
    }

    function test_SetZero() public {
        uint256 number = 0;
        (bytes memory journal, bytes32 post_state_digest, bytes memory seal) = prove(Elf.IS_EVEN_PATH, abi.encode(number));
        evenNumber.set(abi.decode(journal, (uint256)), post_state_digest, seal);
        assertEq(evenNumber.get(), number);
    }
}
```

- **Setup**: For each test, the setUp function is called first to prepare the test environment. This step involves deploying a mock verifier contract or a RISC Zero Verifier contract (depending whether the env variable `RISC0_DEV_MODE` is enable or not), and the EvenNumber contract, ensuring a clean slate for each test case.
- **Off-Chain proof**: The tests use the **prove** function provided by *RiscZeroCheats* to generate a *journal*, *post-state digest*, and *seal* (proof). Under the hood, the `prove` function checks whether the env variable `RISC0_DEV_MODE` is enabled:
    - when enabled it executes the guest program (ELF) locally and returns the journal along with an empty seal. This empty seal will only be accepted by the mock verifier contract.
    - when diabled it uses the Bonsai proving service to run the guest and produce an on-chain verifiable SNARK attesting to the correctness of the journal output.
- **Execution and verification**: With the proof (or mocked) data, the tests call the set function on the EvenNumber contract and then verify that the contract's stored number is updated as expected. This step is where the contract's ability to correctly integrate with RISC Zero proofs and enforce the even number constraint is rigorously checked.
- **Assertion**: Each test concludes with an assertion (assertEq) to compare the expected outcome (the input number) with the actual state of the contract (the stored number). These assertions ensure that the contract behaves correctly in both normal and edge-case scenarios.
# zkVM Methods

This directory contains the [zkVM] portion of your [RISC Zero] application.
This is where you will define one or more [guest programs] to act as a coprocessor to your [on-chain logic].

> In typical use cases, the only code in this directory that you will need to edit is inside [`guest/src/bin`].

[zkVM]: https://dev.risczero.com/zkvm
[RISC Zero]: https://www.risczero.com/
[guest programs]: https://dev.risczero.com/terminology#guest-program
[on-chain logic]: ../contracts/
[`guest/src/bin`]: ./guest/src/bin/

### Writing Guest Code

To learn to write code for the zkVM, we recommend [Guest Code 101].

Examples of what you can do in the guest can be found in the [RISC Zero examples].

[Guest Code 101]: https://dev.risczero.com/zkvm/developer-guide/guest-code-101
[RISC Zero examples]: https://github.com/risc0/tree/v0.18.0/examples

### From Guest Code to Binary File

Code in the `methods/guest` directory will be compiled into one or more binaries.

Build configuration for the methods is included in `methods/build.rs`.

Each will have a corresponding image ID, which is a hash identifying the program.

### Uploading Binary to Bonsai

<!-- TODO: This should have practical instructions on how to actually upload to Bonsai -->

When [deploying] your application, you will upload your binary to Bonsai where the guest will run when requested.
The image ID will be included in the deployment of the smart contracts to reference your guest program living in Bonsai.

[deploying]: ../deployment-guide

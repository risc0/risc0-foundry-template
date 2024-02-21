# Solidity Contracts

This directory contains the Solidity contract for deploying an application with [RISC Zero] on Ethereum.
The example contract included within the template is [EvenNumber.sol]. It holds a number, guaranteed to be even.

Additional contracts get auto-generated when building the project with cargo, by running:

```bash
cargo build
```

or to build guest code for the zkVM target `riscv32im-risc0-zkvm-elf` deterministically:

```bash
RISC0_USE_DOCKER=1 cargo build
```

By setting the env variable `RISC0_USE_DOCKER` a containerized build process via `Docker` will ensure that all builds of your guest code, regardless of the machine or local environment, will produce the same [ImageID]. The ImageID, and its importance to [security], is explained in more detail in our [developer FAQ].

> ***Note:*** *This requires having Docker installed and in your PATH. To install Docker see [Get Docker](https://docs.docker.com/get-docker/).*

Specifically:
- `ImageID.sol`: contains the ImageIDs for the guests implemented in the [methods] directory.
- `Elf.sol`: contains the path of the guest binaries implemented in the [methods] directory. This contract is saved in the `tests` directory in the root of this template.

The Solidity libraries for Bonsai can be found at [github.com/risc0/risc0-ethereum].

Contracts are built and tested with [forge], which is part of the [Foundry] toolkit.
Tests are defined in the `tests` directory in the root of this template.

[Foundry]: https://getfoundry.sh/
[forge]: https://github.com/foundry-rs/foundry#forge
[RISC Zero]: https://risczero.com
[EvenNumber.sol]: ./EvenNumber.sol
[github.com/risc0/risc0-ethereum]: https://github.com/risc0/risc0-ethereum/tree/main/contracts
[methods]: ../methods/README.md
[ImageID]: https://dev.risczero.com/terminology#image-id
[Get Docker]: https://docs.docker.com/get-docker/
[security]: https://dev.risczero.com/faq#security
[developer FAQ]: https://dev.risczero.com/faq#zkvm-application-design

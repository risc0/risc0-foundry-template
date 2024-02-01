# Bonsai Starter CLI

## Guest interface
You can modify the [guest_interface.rs](src/guest_interface.rs) file to implement the `GuestInterface` trait that lets you define how to parse and serialize the guest input and calldata so that your contract can interact with the RISC Zero zkVM and Bonsai. 

## Usage

```bash
Usage: bonsai-starter <COMMAND>

Commands:
  query    Runs the RISC-V ELF binary
  publish  Runs the RISC-V ELF binary on Bonsai and publish the result to Ethererum
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
FROM ubuntu:20.04@sha256:3246518d9735254519e1b2ff35f95686e4a5011c90c85344c1f38df7bae9dd37

# Install Rust and build dependencies
RUN apt-get update
RUN apt-get install -y --no-install-recommends git build-essential ca-certificates clang curl libssl-dev pkg-config ssh
RUN curl --proto '=https' --tlsv1.2 --retry 10 --retry-connrefused -fsSL 'https://sh.rustup.rs' | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /root

# Install foundry through foundryup
RUN curl -L https://foundry.paradigm.xyz | bash; \
    /bin/bash -c 'source $HOME/.bashrc'; \
    /root/.foundry/bin/foundryup
ENV PATH "$PATH:/root/.foundry/bin/"
RUN echo "export PATH=${PATH}" >> $HOME/.bashrc;

# Alternatively, install from source
# RUN cargo install --git https://github.com/foundry-rs/foundry --profile local forge cast chisel anvil

# Copy source code
COPY . /app
WORKDIR /app

# Install rust toolchain
RUN rustup toolchain install .

# Update github host
RUN mkdir -p /root/.ssh && ssh-keyscan github.com > ~/.ssh/known_hosts

# Build solidity code
RUN forge build

# Install risc0 toolchain
RUN cargo install cargo-risczero
RUN cargo risczero install

# Build rust code
RUN --mount=type=ssh  \
    cargo build

ENTRYPOINT ["forge"]

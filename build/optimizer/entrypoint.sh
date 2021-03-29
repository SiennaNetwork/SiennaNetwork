#!/usr/bin/env bash

set -aemux

# The Cargo package that contains the contract
Package=$1
Tag=$2

# Switch to non-root user
USER=${USER:-1000}
GROUP=${GROUP:-1000}
groupadd -g$GROUP $GROUP || true
useradd -m -g$GROUP -u$USER build || true

# The local registry is stored in a Docker volume mounted at /usr/local.
# This sure it is accessible to non-root users, which is the whole point:
mkdir -p /usr/local/cargo/registry
chown -R $USER /usr/local/cargo/registry

echo "Building $Package as user build ($USER:$GROUP)..."

# Execute a release build then optimize it with Binaryen
Output=`echo "$Package" | tr '-' '_'`
ls -alh
su build -c "env RUSTFLAGS='-C link-arg=-s' \
  cargo build -p $Package --release --target wasm32-unknown-unknown --locked --verbose \
  && wasm-opt -Oz ./target/wasm32-unknown-unknown/release/$Output.wasm -o /output/$Tag-$Package.wasm \
  && cd /output/ && sha256sum -b *.wasm > checksums.sha256.txt"

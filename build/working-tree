#!/usr/bin/env bash

# Production build of current HEAD
set -aue
Timestamp=`date --rfc-3339=date`
Commit=`git rev-parse --short HEAD`
for Contract in sienna-mgmt snip20-reference-impl; do
  echo "Now building $Contract:"
  docker run -it --rm                                                    \
    -v "`pwd`":/contract                                                  \
    -e CARGO_NET_GIT_FETCH_WITH_CLI=true                                   \
    -e CARGO_TERM_VERBOSE=true                                              \
    -e CARGO_HTTP_TIMEOUT=240                                                \
    --mount type=volume,source=sienna_cache,target=/code/target               \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    hackbg/secret-contract-optimizer:latest $Contract
  mv "$Contract.wasm" "dist/$Commit-$Contract.wasm"
done

# Generate checksums
cd dist
sha256sum -b *.wasm > checksums.sha256.txt

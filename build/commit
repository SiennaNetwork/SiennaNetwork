#!/usr/bin/env bash

# Production build of arbitrary commit hash
set -aue
cd `dirname $(dirname $(dirname $0))`
pwd
Timestamp=`date --rfc-3339=date`
Commit=${1:-main}
Origin=`git remote get-url origin`
for Contract in sienna-mgmt snip20-reference-impl; do
  echo "Now building $Contract:"
  docker run -it --rm                                       \
    -e CARGO_NET_GIT_FETCH_WITH_CLI=true                     \
    -e CARGO_TERM_VERBOSE=true                                \
    -e CARGO_HTTP_TIMEOUT=240                                  \
    -v "`realpath $(dirname $(dirname $(dirname $0)))`":/output \
    -v $HOME/.ssh/id_rsa:/root/.ssh/id_rsa:ro                    \
    -v $HOME/.ssh/known_hosts:/root/.ssh/known_hosts:ro           \
    --mount type=volume,source=sienna_cache_$Commit,target=/code/target               \
    --mount type=volume,source=registry_cache_$Commit,target=/usr/local/cargo/registry \
    --entrypoint /bin/sh                       \
    hackbg/secret-contract-optimizer:latest     \
    -c "mkdir -p /contract && cd /contract     &&\
        git clone --recursive -n $Origin .      &&\
        git checkout $Commit                     &&\
        git submodule update                      &&\
        chown -R 1000 /contract                    &&\
        /entrypoint.sh $Contract                    &&\
        ls -al && (gzip -df $Contract.wasm.gz||true) &&\
        mv $Contract.wasm /output/"
  ls -al && mv "$Contract.wasm" "dist/$Commit-$Contract.wasm"; done
cd dist
sha256sum -b *.wasm > checksums.sha256.txt

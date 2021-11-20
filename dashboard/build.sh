#!/bin/sh

build () {
  wasm-pack build \
    --dev \
    --target web \
    --out-dir dashboard/artifacts \
    --out-name $1 \
    $2 -- --features browser
}

build sienna  contracts/snip20-sienna
build lptoken contracts/lp-token
build mgmt    contracts/mgmt
build rpt     contracts/rpt
build rewards contracts/rewards

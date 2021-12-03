#!/bin/sh

set -e

cd `dirname $0`/..
echo "Running from `pwd`"

build () {
  echo "Building $1 from $2..."
  wasm-pack build \
    --dev \
    --target web \
    --out-dir "../../dashboard/artifacts/$1" \
    --out-name $1 \
    $2 -- --features browser
}

build sienna  contracts/snip20-sienna

build lptoken contracts/lp-token

build mgmt    contracts/mgmt

build rpt     contracts/rpt

build rewards contracts/rewards

#!/bin/sh

set -e

cd `dirname $0`/..
echo "Running from `pwd`"

build () {
  echo "Building $1 from $2..."
  wasm-pack build \
    --dev \
    --target bundler \
    --out-dir "../../dashboard/artifacts/$1" \
    --out-name $1 \
    $2 -- --features browser
}

build sienna  ../contracts/tge/snip20-sienna

build lptoken ../contracts/amm/lp-token

build mgmt    ../contracts/tge/mgmt

build rpt     ../contracts/tge/rpt

build rewards ../contracts/amm/rewards

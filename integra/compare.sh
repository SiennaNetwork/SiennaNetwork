#!/usr/bin/env bash

# This rebuilds SecretJS so that any changes (such as error handling fixes) are seen by the test.
# If this fails, make sure there is no `scripts/node_modules` directory; if it exists, delete it.
echo "Build SecretJS with sane error handling ----------------------------------------------------"
set -aue
pushd ../platform/cosmwasm-js/packages/sdk
pwd
yarn build
cd ../../../../
pwd
yarn
popd

# This waits for the localnet node to start
echo "Wait for localnet to respond ---------------------------------------------------------------"
./lib/wait.sh localhost 1337

# This runs the actual comparison between versions
echo "Let's compare some builds ------------------------------------------------------------------"
node --trace-warnings --unhandled-rejections=strict ./compare.js

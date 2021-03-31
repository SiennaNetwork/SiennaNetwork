#!/usr/bin/env bash
set -aue

# This rebuilds SecretJS so that any changes (such as error handling fixes) are seen by the test.
# If this fails, make sure there is no `build/node_modules` directory; if it exists, delete it.
echo "Ensuring build deps for SecretJS are available ----------------------------------------------"
pushd platform/cosmwasm-js/packages/sdk
yarn
echo "Building SecretJS with saner error handling -------------------------------------------------"
yarn build
echo "Linking our build of SecretJS into the workspace --------------------------------------------"
cd ../../../../../
pwd
yarn
#echo "Linking contract APIs into the build environment --------------------------------------------"
#cd build
#yarn
popd
echo "Waiting for localnet to respond -------------------------------------------------------------"
./integration/wait.sh localhost 1337
echo "Now running:\n$@"
node --trace-warnings --unhandled-rejections=strict $@

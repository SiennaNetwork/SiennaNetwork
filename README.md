# SIENNA

## Quick start

Here's how to fetch the code, install JS dependencies,
and obtain a list of the actions you can perform:

```sh
git clone --recurse-submodules git@github.com:hackbg/sienna-secret-token.git sienna
cd sienna
yarn
./sienna --help
```

## Integration testing

Integration testing is done through javascript `mocha` tool, tests are located in the `api/` directory together with
contract wrappers that enable you easy usage of them.

Tests should automatically lift the docker container that will hold the localnet instance from which you can
have working local blockchain where the contracts will be deployed. Each test will handle lifting of the image,
running the tests and all the required setup, and then will terminate the container so the other test can do the same.

This will ensure we have a clean slate each time contracts tests are run.

### Rewards

Running the rewards test can be done directly by calling the `mocha` command:

```
mocha -p false api/Rewards.spec.js
```

Rewards have their wrapper located in `api/Rewards.js` which will instantiate the new contract on any given network.

Please review the `api/Rewards.spec.js` for more detailed clarification of the instantiation process.

#### SNIP20

Located under `api/SNIP20.js` you'll find the wrapper for any `snip20` contract that will expose all the required methods to call on the contract.

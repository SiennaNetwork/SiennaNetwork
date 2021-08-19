# SIENNA

## Mainnet addresses

|Contract  |Address<br>Code hash|
|:---------|:-------------------|
|**SIENNA**|**secret1rgm2m5t530tdzyd99775n6vzumxa5luxcllml4**<br>c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084|
|**MGMT**  |**secret1kn6kvc97nvu69dqten0w9p9e95dw6d6luv3dfx**<br>b1e4c4d76a5aedd180d08d8fec99ad84ed1a8a08d6d8a32a30c8c0f9835f4fab|
|**RPT**   |**secret107j8czcysrkvxsllvhqj4mhmcegt9hx2ra3x42**<br>a9bfc78d182eb8d3cbb74d4269ef1f529a607f7842d755f00fef7df13c02c5b4|
|**SIENNA on BSC**|**0x130F6E4d338BFD8304F5342D759ABE5C6Bd7bA9b**|N/A|
|**wSIENNA on ETH**|**0x9b00e6E8D787b13756eb919786c9745054DB64f9**|N/A|

## Quick start

Here's how to fetch the code, install JS dependencies,
and obtain a list of the actions you can perform:

```sh
git clone --recurse-submodules git@github.com:hackbg/sienna-secret-token.git sienna
cd sienna
yarn
./sienna --help
```

>⚠️ **NOTICE:** This requires **Yarn 2** (Berry). Switching between Yarn versions
>may overwrite `.yarnrc.yml`; use `git checkout yarnrc.yml` to restore it.

>⚠️ **NOTICE:** If Yarn fails, make sure you've initialized the **submodules**.
>If you didn't clone with `--recurse-submodules`, you can use
>`git submodule init && git submodule update`.

## Integration testing

Integration testing is done through javascript `mocha` tool,
tests are located in the `api/` directory together with
contract wrappers that enable you easy usage of them.

Tests should automatically lift the docker container
that will hold the localnet instance from which you can
have working local blockchain where the contracts will be deployed.
Each test will handle lifting of the image, running the tests and
all the required setup, and then will terminate the container so
the other test can do the same.

This will ensure we have a clean slate each time contracts tests are run.

### Rewards

Running the rewards test can be done directly by calling the `mocha` command:

```
mocha -p false api/Rewards.spec.js
```

Rewards have their wrapper located in `api/Rewards.js` which
will instantiate the new contract on any given network.

Please review the `api/Rewards.spec.js` for more detailed
clarification of the instantiation process.

#### SNIP20

Located under `api/SNIP20.js` you'll find the wrapper for any `snip20` contract
that will expose all the required methods to call on the contract.

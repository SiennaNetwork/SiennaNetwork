# SIENNA

## Mainnet addresses

|Contract  |Address<br>Code hash|
|:---------|:-------------------|
|**SIENNA**|**`secret1rgm2m5t530tdzyd99775n6vzumxa5luxcllml4`**<br>`c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084`|
|**MGMT**  |**`secret1kn6kvc97nvu69dqten0w9p9e95dw6d6luv3dfx`**<br>`b1e4c4d76a5aedd180d08d8fec99ad84ed1a8a08d6d8a32a30c8c0f9835f4fab`|
|**RPT**   |**`secret107j8czcysrkvxsllvhqj4mhmcegt9hx2ra3x42`**<br>`a9bfc78d182eb8d3cbb74d4269ef1f529a607f7842d755f00fef7df13c02c5b4`|
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

## Contents

|Environment|Component     |MGMT|RPT|Rewards|AMM|IDO|
|----|---------------------|----|---|-------|---|---|
|Rust|Smart contract(s)    |✔️   |✔️  |✔️      |✔️  |✔️  |
|Rust|Unit tests           |    |   |✔️      | ? | ? |
|JS  |API wrapper(s)       |✔️   |✔️  |✔️      |✔️  |✔️  |
|JS  |API integration tests|✔️   |✔️  |       | ? | ? |
|JS  |Gas benchmark        |    |   |✔️      | ? | ? |
|JS  |Dashboard            |    |   |✔️      | ? | ? |

### Smart contracts

The smart contracts are written in Rust targeting
SecretNetwork's fork of `cosmwasm-std 0.10.*`.

To obtain a production build of Sienna Rewards:

```sh
./sienna rewards build
```

### Unit tests

These tests cover the business logic of the contract
in a mocked out environment. They run relatively quickly,
and output any compilation errors, which makes them perfect
for iterating on contracts.

In the case of Sienna Rewards, the unit tests are two-tiered:
`rewards_test.rs` tests the contract through its public API, while
`rewards_test_2.rs` tests the underlying business logic implementation
by calling the internal methods directly. To run both:

```sh
./sienna rewards test
```

### API wrappers, integration tests, and benchmarks

JS/TS modules for deploying and operating the contracts can be found
in `api/`, `ops/`, and `ops/amm-lib/`. The ones in `api/` and `ops/`
are based on Fadroma (`Contract` and `Ensemble` classes),
while the ones in `ops/amm-lib/` invoke SecretJS directly.

The API integration tests are based on [Mocha](https://mochajs.org/).

Fadroma provides a "localnet" container - an ephemeral local blockchain
that is set up and torn down between test cases.

To run the Sienna Rewards benchmark:

```sh
./sienna localnet reset
./sienna rewards benchmark
```

### Dashboard

TODO more info

## See also

### SNIP20

Located under `api/SNIP20.js` you'll find the wrapper for any `snip20` contract
that will expose all the required methods to call on the contract.

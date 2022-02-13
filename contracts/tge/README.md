# Sienna TGE/Vesting

## Contents

* `snip20-sienna` - Core SIENNA token for Secret Network
* `wrapped` - SIENNA token wrapped as ERC20 (Ethereum)
* `mgmt` - Main vesting management contract
* `rpt` - Remaining pool token distribution contract

## Compile for production

```sh
pnpm -w dev build tge
```

## Run tests

```sh
cargo test -p snip20-sienna
cargo test -p mgmt
cargo test -p rpt
```

## Deploying `wSIENNA` on Ethereum

<table>
<tr><td>

Use these commands to deploy `wSIENNA` on Ethereum mainnet

```bash
# clone the project
git clone https://github.com/SiennaNetwork/wrapped-sienna

# install the dependencies
npm install

# compile the contract
npx truffle compile

# test the contract
npx truffle test

# choose `mainnet` for mainnet deployment and `rinkeby` for testnet
npx truffle migrate --network <network>
```

* Place the deploying seed in a `.secret` file
  in `/contracts/tge/wrapped`.

* Make sure the account has enough funds for deployment.

* In `migrations/2_deploy_contracts.js`,
  make sure the address for the **bridge** is the correct one
  for the network you are planning to deploy the contracts on
  (see below)

</td><td>
</td></tr>
</table>

### Addresses of SCRT-ETH bridges

| Network           | SCRT-ETH Bridge Address                        |
| ----------------- | ---------------------------------------------- |
| Rinkeby (testnet) | **0xFA22c1BF3b076D2B5785A527C38949be47Ea1082** |     |
| Mainnet           | **0xf4b00c937b4ec4bb5ac051c3c719036c668a31ec** |   |

## Deploying `SIENNA` on Secret Network

### Preparing the deploy wallet

<table>
<tr><td>

* Make sure that you have access to a mainnet wallet with
  at least **8 SCRT**. We'll call this the **deploy wallet**.

* Make sure the deploy wallet is added to `secretcli`.

</td><td>

Use this command to add a wallet to `secretcli`:

```bash
secretcli keys add DeploySIENNA --recover
# enter mnemonic
```

Use this command to see the address for a wallet:

```bash
secretcli keys show -a DeploySIENNA
```

Use this command to create a new wallet:

```bash
secretcli keys add DeploySIENNA
# make sure to save the generated mnemonic!
```

</td></tr>
</table>

### Deploying the TGE contracts

<table>
<tr><td>

* Make sure you have prepared the correct vesting schedule
in `settings/schedule.ods`.

* Make sure you're running in a secure environment
  with a reliable Internet connection.

* Make sure you have access to the following tools in your
  environment:
  - Bash
  - Git
  - jq
  - Docker
  - Node.JS + Yarn
  - secretcli

</td><td>

> **FIXME:** These commands are out of date.

```bash
# 1. clone the repo
git clone --recursive https://github.com/SiennaNetwork/sienna-secret-token

# 2. Install dependencies
yarn

# 3. Configure environment
export SECRET_NETWORK_MAINNET_ADDRESS='your address'
export SECRET_NETWORK_MAINNET_MNEMONIC='your mnemonic'

# 5. Run sienna deploy
./sienna.js deploy mainnet # testnet for holodeck-2

# 5. Remove your mnemonic from the environment immediately afterwards!
export SECRET_NETWORK_MAINNET_MNEMONIC=
```

> ℹ️  Replace `mainnet` with `testnet` to deploy on the holodeck-2 testnet (via our hardcoded test account, `secret1vdf2hz5f2ygy0z7mesntmje8em5u7vxknyeygy`).

</td></tr>

<tr><!--spacer--></tr>

<tr><td>

If the deployment succeeds, you should see a table in your terminal,
containing the addresses and code hashes needed to interface with the deployed contracts.

* Give the TOKEN address/hash to users of the token,
* Give the MGMT address/hash to recipients of the vesting.

>ℹ️ This table is extracted from the full build results, which are stored in `./artifacts/checksums.sha256.txt`, `./artifacts/<chain-id>/uploads/` and `./artifacts/<chain-id>/instances/`.

</td><td>

| Contract<br>Description | Address<br>Code hash |
|-|-|
|**TOKEN**<br>Sienna SNIP20 token |...|
|**MGMT**<br>Vesting |...|
|**RPT**<br>Remaining pool tokens |...|

</td></tr>
</table>

### Transfering ownership to the multisig account

<table>
<tr><td>

You should now have 3 contracts:
* **TOKEN**
* **MGMT**
* **RPT**

They are owned by the deployer wallet.

They are all interconnected:
* MGMT is admin and sole minter of TOKEN
* MGMT and RPT point to each other

Hence, to transfer control over them to the multisig account,
you need to do perform the following steps:

</td><td>

</td></tr>

<tr><!--spacer--></tr>

<tr><td>

##### Transfer ownership of MGMT

```bash
secretcli tx compute execute MGMT_ADDRESS \
  '{"set_owner":{"new_admin":"MULTISIG_ADDRESS"}}' \
  --from DEPLOYMENT_KEY \
  --chain-id secret-2 \
  --gas 450000
```

> Replace `secret-2` with `holodeck-2` if deploying to testnet)

</td><td>

Example:
```bash
# Transfer ownership
secretcli tx compute execute secret1f6g9aunzcucdpzmnsq759ucz2vhv8psmfcquvv \
 '{"set_owner":{"new_admin":"secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a"}}' \
 --from alabala
 --chain-id holodeck-2
 --gas 450000
# Confirm the transaction succeeded
secretcli q compute tx 1EDFF76280B80FE548FDBBA4F64F684F1C4A9EA3F8EC882ED7E3BE77D5A5421A | jq '.'
```

Example output:
```json
{
  "type": "execute",
  "raw_input": "a9bfc78d182eb8d3cbb74d4269ef1f529a607f7842d755f00fef7df13c02c5b4{\"set_owner\":{\"new_admin\":\"secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a\"}}",
  "input": null,
  "output_data": "",
  "output_data_as_string": "",
  "output_log": [],
  "output_error": {},
  "plaintext_error": ""
}
```

</td></tr>

<tr><!--spacer--></tr>

<tr><td>

##### Transfer ownership of RPT

```bash
secretcli tx compute execute RPT_ADDRESS \
  '{"set_owner":{"new_admin":"MULTISIG_ADDRESS"}}' \
  --from DEPLOYMENT_KEY \
  --chain-id secret-2 \
  --gas 450000
```
(Replace `secret-2` with `holodeck-2` if deploying to testnet)

</td><td>

Example:
```bash
# Transfer ownership
secretcli tx compute execute secret1ayr3226h2xkzr59juw8cq2v5wt7cuc3cmvfn6e \
 '{"set_owner":{"new_admin":"secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a"}}' \
 --from alabala \ 
 --chain-id holodeck-2 \
 --gas 450000
# Confirm the transaction succeeded
secretcli q compute tx 233C65A6370EA7B037E14996B6158659078AA90727924A63D0458DB39F96DEC0 | jq '.'
```

Example output:

```json
{
  "type": "execute",
  "raw_input": "a9bfc78d182eb8d3cbb74d4269ef1f529a607f7842d755f00fef7df13c02c5b4{\"set_owner\":{\"new_admin\":\"secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a\"}}",
  "input": null,
  "output_data": "",
  "output_data_as_string": "",
  "output_log": [],
  "output_error": {},
  "plaintext_error": ""
}
```

</td></tr>

<tr><!--spacer--></tr>

</table>

## Configure

* MGMT can be reconfigured by its admin after deployment
  as long as it hasn't been launched yet.

* RPT can be freely reconfigured by its admin
  as long as the budget adds up to the original amount (2500 SIENNA).

## Use

* To claim funds from MGMT, send it `{"claim":{}}`.
* To make RPT send funds to the reward pools, send it `{"vest":{}}`

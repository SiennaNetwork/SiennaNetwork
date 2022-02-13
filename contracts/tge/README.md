# Sienna TGE/Vesting

* [Contents](#contents) of this directory
* [Compile for production](#compile-for-production)
* [Run tests](#run-tests)
* [Deploy `wSIENNA` on Ethereum](#deploy-wsienna-on-ethereum)
  * [Addresses of SCRT-ETH bridges](#addresses-of-scrt-eth-bridges)
* [Deploy `SIENNA` on Secret Network](#deploy-sienna-on-secret-network)
  * [Prepare the deploy wallet](#prepare-the-deploy-wallet)
  * [Deploy the TGE contracts](#deploy-the-tge-contracts)
  * [Transfer ownership to multisig account](#transfer-ownership-to-multisig-account)
* [Configure](#configure)
* [Use](#use)

* [Emergency mode](#Emergency-mode)
  * Pausing contract
  * Resuming normal operation
  * Deploying an updated version of the contract
  * Pausing all transactions with the token
* [Launch vesting](#Launch-Vesting)
  * [Adding a new account to vesting](#Adding-a-new-account-to-vesting)

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

## Deploy `wSIENNA` on Ethereum

<table>
<tr><td>

* Place the deploying seed in a `.secret` file
  in `/contracts/tge/wrapped`.

* Make sure the account has enough funds for deployment.

* In `migrations/2_deploy_contracts.js`,
  make sure the address for the **bridge** is the correct one
  for the network you are planning to deploy the contracts on
  (see below)

</td><td>

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

</td></tr>
</table>

### Addresses of SCRT-ETH bridges

| Network           | SCRT-ETH Bridge Address                        |
| ----------------- | ---------------------------------------------- |
| Rinkeby (testnet) | **0xFA22c1BF3b076D2B5785A527C38949be47Ea1082** |     |
| Mainnet           | **0xf4b00c937b4ec4bb5ac051c3c719036c668a31ec** |   |

## Deploy `SIENNA` on Secret Network

### Prepare the deploy wallet

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

### Deploy the TGE contracts

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

### Transfer ownership to multisig account

You should now have 3 contracts owned by the deployer wallet:
**TOKEN**, **MGMT**, and **RPT**.

They are all interconnected:
* MGMT is admin and sole minter of TOKEN
* MGMT and RPT point to each other

To transfer control over them to the multisig account,
you need to do perform the following steps:

<table>
<tr><td valign="top">

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

<tr><td valign="top">

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

</table>

## Configure

* MGMT can be reconfigured by its admin after deployment
  as long as it hasn't been launched yet.

* RPT can be freely reconfigured by its admin
  as long as the budget adds up to the original amount (2500 SIENNA).

<table>
<tr><td>

### Read the current vesting schedule

```bash
secretcli q compute query secret1kn6kvc97nvu69dqten0w9p9e95dw6d6luv3dfx '{"schedule":{}}'
```

</td><td>

Example output:

```jsonc
{
   "schedule":{
      "schedule":{
         "total":"10000000000000000000000000",
         "pools":[
            {
               "partial":true,
               "name":"Investors",
               "total":"2000000000000000000000000",
               "accounts":[
                  {
                     "name":"978",
                     "address":"secret1leulrux3emu7c34jux0n8x0v6y9cfhl4k8xk08",
                     "amount":"155000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  }
                  // ...
               ]
            },
            // ...
            {
               "partial":false,
               "name":"MintingPool",
               "total":"3900000000000000000000000",
               "accounts":[
                  {
                     "name":"LPF",
                     "address":"secret1wdhvhe0wd5ufhx4jwfv29se74u45m2xjkqm2ld",
                     "amount":"300000000000000000000000",
                     "cliff":"300000000000000000000000",
                     "start_at":0,
                     "interval":0,
                     "duration":0
                  },
                  {
                     "name":"RPT",
                     "address":"secret107j8czcysrkvxsllvhqj4mhmcegt9hx2ra3x42",
                     "amount":"3600000000000000000000000",
                     "cliff":"0",
                     "start_at":0,
                     "interval":86400,
                     "duration":124416000
                  }
               ]
            }
         ]
      }
   }
}
```

</td></tr>
<tr><!--separator--></tr>

<tr><td>

### Add a new account to vesting

Send this message in a transaction from the admin address to the MGMT contract to unlock 1 SIENNA immediately for address `secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a`:

```json
{"add_account":{"pool":"Investors","account":{"name":"someone","amount":"1000000000000000000","address":"secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a","start_at":0,"interval":0,"duration":0,"cliff":"1000000000000000000"}}
```

</td><td>

Example:
```bash
# Sample transaction to add 1 SIENNA with immediate vesting to sample account
secretcli tx compute execute MGMT_CONTRACT_ADDRESS '{"add_account":{"pool_name":"Investors","account":{"name":"someone","amount":"1000000000000000000","address":"secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a","start_at":0,"interval":0,"duration":0,"cliff":"1000000000000000000"}}}' --from ADMIN_KEY_ALIAS --chain-id NETWORK_ID --gas 450000
```

>ℹ️ `start_at`, `interval`, `duration` are in seconds (1 day = 86400 seconds). For immediate vesting, set them all to 0 and `cliff` = `amount`. 

>ℹ️ `amount` and `cliff` are in attoSIENNA (multiply by `1000000000000000000` - 1 with 18 zeros - to get SIENNA), and must be in double quotes (`"`) - because JSON doesn't support numbers that big.

>⚠️ Be careful - errors here are permanent and can't be remedied without a full migration (untested procedure!)

</td></tr>

</table>

<table>
<tr><td>

### Launch the vesting

Launch the vesting with this message.

Configure and sign the transaction the multisig transaction,
just the receiving contract should be MGMT (which you have deployed already).

Transaction message: 

```bash
{"launch":{}}
```

</td><td>

Example:
```bash
# Start vesting
secretcli tx compute execute MGMT_CONTRACT_ADDRESS '{"launch":{}}' --from ADMIN_KEY_ALIAS --chain-id NETWORK_ID --gas 450000
```

> ℹ️ NETWORK_ID is holodeck-2 for testnet & secret-2 for mainnet
```
# Confirm the transaction succeeded
secretcli q compute tx TRANSACTION_HASH | jq '.'
```

</td></tr>
</table>

## Use

* To claim funds from MGMT, send it `{"claim":{}}`.
* To make RPT send funds to the reward pools, send it `{"vest":{}}`

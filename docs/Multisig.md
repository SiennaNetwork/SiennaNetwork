# SIENNA - Multisig, deployment and setup

*Note: Because of the recently found issues with instantiating
multisig contracts via the official Secret Network CLI,
the setup below will outline how to deploy and instantiate the contracts
for the Sienna TGE from a regular (single) account,
and then transfer the ownership of those to a multisig account of choice,
which can later be used for execution of different admin actions.*

# Table of contents

* [Ethereum](#ETHEREUM)
    * [Deploying WSIENNA ERC20](#Deploying-WSIENNA-wrapped-SIENNA---ERC20)
        * [Deploying on mainnet](#Deploy-on-mainnet)
* [Secret Network](#SECRET-NETWORK)
    * [Before deploying](#Before-deploying)
    * [Deploying](#Deploying)
        * [Transfering ownership to the multisig account](#Transfering-ownership-to-the-multisig-account)
            * [Add deployment key to secretcli](#Add-deployment-key-to-secretcli)
            * [Transfer ownership](#Transfer-ownership)
                * [MGMT - Vesting Contract](#MGMT)
                * [RPT - Remaining Pool Tokens Contract](#RPT)
    * [Multisig executing actions](#Multisig-executing-actions)
        * [Create a multisig account](#Create-a-multisig-account)
            * Creating the multisig account locally
            * Creating the multisig account
        * [Prepare, sign, and broadcast multisig transactions](#Prepare-sign-and-broadcast-multisig-transactions)
            * Get the required certificates
            * Prepare offline transaction for the multisig account
            * Distribute for signatures
            * Individual signing
            * Preparing signed transaction using the collected signatures
            * Broadcasting the tx
        * [Emergency mode](#Emergency-mode)
            * Pausing contract
            * Resuming normal operation
            * Deploying an updated version of the contract
            * Pausing all transactions with the token
         * [Launch vesting](#Launch-Vesting)
            * [Adding a new account to vesting](#Adding-a-new-account-to-vesting)


## Multisig executing actions

### Create a multisig account

1. Creating the multisig account locally

In order to create the account first you need to collect all of the relevant participating parties' public keys.

*Hint: You can also refer to the official secretcli documentation as a helper [here](https://build.scrt.network/validators-and-full-nodes/secretcli.html#multisig-transactions).*

In the example bellow we'd assume you already have your key imported in the `secretcli` with an alias of preference:

```bash
# import the first participant's pubkey with the alias `participant2`
secretcli keys add participant2 --pubkey=secretpub1addwnpepqtd28uwa0yxtwal5223qqr5aqf5y57tc7kk7z8qd4zplrdlk5ez5kdnlrj4

# import the first participant's pubkey with the alias `participant3`
secretcli keys add participant3 --pubkey=secretpub1addwnpepqgj04jpm9wrdml5qnss9kjxkmxzywuklnkj0g3a3f8l5wx9z4ennz84ym5t 
```

2. Creating the multisig account

Use the aliases of the imported accounts to form a new multisig account.

Set up the desired threshold of required signatures for a valid transaction. In the example below threshold of 2 would mean 2 singatures from all of the multisig parties are enough to create and execute a valid transaction.

```bash
# create the multisig account with alias `MULTISIG_ACCOUNT`
secretcli keys add \
  MULTISIG_ACCOUNT \
  --multisig-threshold=2 \
  --multisig=MY_ACCOUNT_ALIAS,participant2,participant3
```

```bash
# To confirm the above command worked:
secretcli keys show MULTISIG_ACCOUNT -a
# As output you should see the multisig account address
```

### Prepare, sign, and broadcast multisig transactions

#### 1. Get the required certificates

These certificates are needed for encryption of the tx data with the node's secure enclave.

```bash
secretcli query register secret-network-params
```

*The command above will download the needed certificates in the current directory, make sure you proceed further within the same directory or address the path to the certificate properly in the next steps.*

#### 2. Prepare offline transaction for the multisig account

In the example below we will demonstrate how to prepare a multisig transaction that executes a smart contract method. For example minting tokens from a SNIP20 contract via the `mint` method, assuming the contract is owned by the multisig and the multisig account has minter access granted.


```bash
# Get multisig sequence and account number
secretcli q account MULTISIG_ACCOUNT_ADDRESS
```

The command above will give you the information about `MULTISIG_ACCOUNT_SEQUENCE` and `MULTISIG_ACCOUNT_NUMBER` that you will need in order to proceed further and succesfully create an offline multisig tx.

Example output:

```json=
{
  "type": "cosmos-sdk/Account",
  "value": {
    "address": "secret1gerc2dh5mxejushnn3vs9j4l09z800y8dduwf9",
    "coins": [
      {
        "denom": "uscrt",
        "amount": "20000000"
      }
    ],
    "public_key": "",
    "account_number": 9276,
    "sequence": 0
  }
}
```

Continue with preparation of the transaction towards the `SMART_CONTRACT_ADDRESS`, containing the message that we want to send.

```bash
# Prepare the offline transaction
secretcli tx compute execute SMART_CONTRACT_ADDRESS \
'{"mint": {"recipient": "RECIPIENT_ADDRESS", "amount": "42000000000000000000" }}' \
--generate-only \
--chain-id secret-2 \
--gas 450000 \
--from MULTISIG_ACCOUNT_ADDRESS \
--enclave-key io-master-cert.der \
--code-hash CONTRACT_CODE_HASH \ # you should have that from the contract deployment. stored if you filled the table above
--sequence MULTISIG_ACCOUNT_SEQUENCE \
--account-number MULTISIG_ACCOUNT_NUMBER \
> unsignedTx.tx
```

If everything went well in the output file which we named `unsignedTx.tx` you should have an output similar to the one below:

```json=
{
  "type": "cosmos-sdk/StdTx",
  "value": {
    "msg": [
      {
        "type": "wasm/MsgExecuteContract",
        "value": {
          "sender": "secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a",
          "contract": "secret1wgeqzh7dd7l2wwllyky0dh052hmzsp903fhfnj",
          "msg": "0e10K13qPzYt7M+8RPzdbXwQbZExR0k5ZELM3n4DFKO4V+4MhX0kX5ekh7w12sz85tQz2OLjcloMrPSeur8xMOaNRY04zxp1CTTszwh3qA/bndYjTRJOqvj1tvOT9OyFeAApxRGrJgUPztNQlAb/8uzy4VSbxAm291oVy69FYOxzRorJOSyWDMW7g3UTzI6BvarSWz4ppH8N7NNKqWgQIXL+qlx4cwGxTLcqrn/XewQann7o53iPkxVUWgKm6Kp9QYysE8+hoQXhAbn/b1jUkbjv7LWGjr6RUdqWTjsQ5WSB1GiyJ1UJSEL+D/g2GVxKF4BguQHcBuB1",
          "callback_code_hash": "",
          "sent_funds": [],
          "callback_sig": null
        }
      }
    ],
    "fee": {
      "amount": [
        {
          "denom": "uscrt",
          "amount": "112500"
        }
      ],
      "gas": "450000"
    },
    "signatures": null,
    "memo": ""
  }
}
```

#### 3. Distribute for signatures

Now that we have the prepared transaction its time to collect the signatures required. Depending on the threshold that was set for the multisig - you can distribute the file `unsignedTx.tx` to the required amount of signers by any means of communication (IM, email, etc.).

#### 4. Individual signing

Each of the signers should individually sign the file as so:

```bash
secretcli tx sign unsignedTx.tx \
--multisig=MULTISIG_ADDRESS \
--offline \
--account-number=MULTISIG_ACCOUNT_NUMBER \
--sequence=MULTISIG_ACCOUNT_SEQUENCE \
--chain-id=secret-2 \
--from=ACCOUNT_ALIAS \
--output-document=mint_signed_participant_N.json
```

>ℹ️ name the different files with the number of the signer for conveinience. 

Example file output:

```json=
{
  "pub_key": {
    "type": "tendermint/PubKeySecp256k1",
    "value": "AusMnK+0hQCDvZZ8QPNXazpA2NCy1/WoTlgUMyVOJZxI"
  },
  "signature": "jAisaC3E91B0d49KMSxOLKH9gsI5Rq05ZCc4sNWXz1990OBui0flUisY6PRuv48g75rzTS0CloyGbk+x23LqKg=="
}
```

#### 5. Preparing signed transaction using the collected signatures

After the signatures has been collected you should have a set of multiple files containing signatures for the accounts participating in the multisig and the `unsignedTx.tx` file that contains the transaction information.

```bash
# Multisign the transaction with the collected signatures
secretcli tx multisign unsignedTx.tx \
 MULTISIG_ALIAS \
 mint_signed_participant1.json \
 mint_signed_participant2.json \
 --offline \
 --account-number=MULTISIG_ACCOUNT_NUMBER \
 --sequence=MULTISIG_ACCOUNT_SEQUENCE \
 > signedTx.json
```

Example output content of the `signedTx.json` file:
```json=
{
  "type": "cosmos-sdk/StdTx",
  "value": {
    "msg": [
      {
        "type": "wasm/MsgExecuteContract",
        "value": {
          "sender": "secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a",
          "contract": "secret1wgeqzh7dd7l2wwllyky0dh052hmzsp903fhfnj",
          "msg": "0e10K13qPzYt7M+8RPzdbXwQbZExR0k5ZELM3n4DFKO4V+4MhX0kX5ekh7w12sz85tQz2OLjcloMrPSeur8xMOaNRY04zxp1CTTszwh3qA/bndYjTRJOqvj1tvOT9OyFeAApxRGrJgUPztNQlAb/8uzy4VSbxAm291oVy69FYOxzRorJOSyWDMW7g3UTzI6BvarSWz4ppH8N7NNKqWgQIXL+qlx4cwGxTLcqrn/XewQann7o53iPkxVUWgKm6Kp9QYysE8+hoQXhAbn/b1jUkbjv7LWGjr6RUdqWTjsQ5WSB1GiyJ1UJSEL+D/g2GVxKF4BguQHcBuB1",
          "callback_code_hash": "",
          "sent_funds": [],
          "callback_sig": null
        }
      }
    ],
    "fee": {
      "amount": [
        {
          "denom": "uscrt",
          "amount": "112500"
        }
      ],
      "gas": "450000"
    },
    "signatures": [
      {
        "pub_key": {
          "type": "tendermint/PubKeyMultisigThreshold",
          "value": {
            "threshold": "2",
            "pubkeys": [
              {
                "type": "tendermint/PubKeySecp256k1",
                "value": "A3/Q8RYuU+ZhDr4CUK4sXbHTJmfVfS7dSSvIPGnV0NPM"
              },
              {
                "type": "tendermint/PubKeySecp256k1",
                "value": "A4dNqo9MTkhWzPMylM4P1C+yQMy2D0hfyTxBaobfqKW7"
              },
              {
                "type": "tendermint/PubKeySecp256k1",
                "value": "AusMnK+0hQCDvZZ8QPNXazpA2NCy1/WoTlgUMyVOJZxI"
              }
            ]
          }
        },
        "signature": "CgUIAxIBoBJAFz0znTHsesC2m+b63Lyt+2AI5P6pxaYuoCV1hah6cfhFtUhh8f/69q5t9EzJyCBoBHymRdsHB//D03SoEQY/YhJAjAisaC3E91B0d49KMSxOLKH9gsI5Rq05ZCc4sNWXz1990OBui0flUisY6PRuv48g75rzTS0CloyGbk+x23LqKg=="
      }
    ],
    "memo": ""
  }
}
```

#### 6. Broadcasting the tx

Now all that is left is to broadcast the transaction to the network.

```bash
secretcli tx broadcast signedTx.json
```

As a response you will see the transaction hash similar to this:

```json
{"height":"0","txhash":"1DBFA76575F52C2C45D61432912E7376C7C9C2B43ACB545DA9DE1757F1E5E529","raw_log":"[]"}
```

Then you can verify the transaction was executed succesfully via the following:

```bash
secretcli q compute tx TX_HASH
```

If all went well you should see similar output:

```json=
{
  "type": "execute",
  "raw_input": "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084{\"mint\":{\"recipient\":\"secret14rn2yd4wke7gfu7tga6l37k4lz2p8rh8chp07z\", \"amount\": \"20000000000000000000\" }}",
  "input": null,
  "output_data": "eyJtaW50Ijp7InN0YXR1cyI6InN1Y2Nlc3MifX0gICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgIA==",
  "output_data_as_string": "{\"mint\":{\"status\":\"success\"}}                                                                                                                                                                                                                                   ",
  "output_log": [],
  "output_error": {},
  "plaintext_error": ""
}
```

## Emergency mode

In case of unexpected events,
the admin of the contracts
(normally, the multisig wallet)
can send one of the following transactions
to allow for manual recovery.

### Pausing contract 

Transaction message:

```json
{"set_status":{"level":"Paused","reason":"This contract is paused because someone did a silly thing"}}
```

Sending this message to MGMT or RPT would pause that contract.
Any transactions will return an error message containing the specified reason.

>ℹ️ Note that you can do this separately for each of the two.
>If you pause MGMT, RPT will be unable to claim any additional funds from it.
>If you pause RPT, MGMT continues operating normally.

### Resuming normal operation

Transaction message:

```json
{"set_status":{"level":"Operational","reason":""}}
```

This returns a paused contract to its `Operational` state,
allowing transactions to proceed normally.

>ℹ️ When MGMT or RPT is paused, time continues to pass for the vesting.
>When it is resumed, users will be able claim the funds accumulated in the meantime.
      
### Deploying an updated version of the contract

Transaction message:

```json
{"set_status":{"level":"Migrating","reason":"He's dead, Jim"}}
```

This pauses the contract **permanently**. Once you've deployed an updated version,
you can send another `set_status` to provide the new contract address
that will be presented to users in the error message.

Transaction message: 

```json
{"set_status":{"level":"Migrating","reason":"Fixed!","new_address":"secret1....."}}
```

>ℹ️ Actual migrations are manual.
>Other than permanently stopping the contract and telling users the new address,
>the `Migrating` state performs no other operations. Future versions may implement
>automated migration logic as needed.

### Pausing all transactions with the token

The TOKEN has its own built-in pause mechanism. To switch the TOKEN to the equivalent of emergency mode,
you need to first switch MGMT to `Migrating`.

Besides disabling MGMT, this will transfer
admin rights on the token (which are normally held by MGMT itself)
to the admin of MGMT (normally, the multisig wallet).

Then, the admin needs to send the following message in a transaction to the token contract:

```json
{"set_contract_status":{"level":"StopAll"}}
```

>⚠️ Migrating TOKEN balances is currently not supported, as exporting them en masse is
>not possible with the Secret Network privacy model. A future SNIP20 implementation could allow
>token holders to privately migrate their balances on a self-serve basis.

>ℹ️ As the `Redeem` SNIP20 method is not part of Sienna's model,
>the `StopAllButRedeems` level that you might see in other SNIP20 implementations
>is not supported.



## Launch Vesting

Launching the vesting with this message. Configure and sign the transaction the same way as above, just the receiving contract should be MGMT (which you have deployed already).

Transaction message: 

```bash
{"launch":{}}
```

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

### Reading current vesting schedule

```bash
secretcli q compute query secret1kn6kvc97nvu69dqten0w9p9e95dw6d6luv3dfx '{"schedule":{}}'
```
```json
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
                  },
                  {
                     "name":"469",
                     "address":"secret1f9dvdd0pz347hshpdgfykf5vwg8a8cf4xrswae",
                     "amount":"80000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"62",
                     "address":"secret1m0j72r2jg8wey2uhwajamlwh74dzu5qj038jt5",
                     "amount":"50000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"194",
                     "address":"secret1jhraz4ftxl2pd37knmeua7wjxghmlskw6cumj9",
                     "amount":"16000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"313",
                     "address":"secret1r3tup72x6693lmpe2kg08dgxyg9490v5e6scwd",
                     "amount":"110000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"201",
                     "address":"secret189lz27yf9xljnfqgcdq4pml7s30zz28c9ndjan",
                     "amount":"50000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"347",
                     "address":"secret1r8lf4lmx6lnza7ssjdcsqdqggwxwrhufl6eryx",
                     "amount":"10000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"266",
                     "address":"secret1qz7nkw437wln83q4zypey572x6g8uk7wnnplya",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"367",
                     "address":"secret1yd87qfnepfn7x70t78tkvdam7sqtu8zzyterny",
                     "amount":"40000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"551",
                     "address":"secret17cmgc4skqcf2pxuqk4xjad2xrn35yjv4aruu50",
                     "amount":"40000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"733",
                     "address":"secret19csp3fk27v9w6hn0wqdq4sycf9skcu0znulnjt",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"391",
                     "address":"secret149zn4ghv6s68xujm5y0whtfn47xcd9r0aaes2m",
                     "amount":"50000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"883",
                     "address":"secret1j4dkn9ghl5t7n8w6xhw8aefhgegr37y6cm8kj5",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"780",
                     "address":"secret1vr77umdykwwtx9dmewhtfj0cd6ur09358u8c7s",
                     "amount":"60000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"175",
                     "address":"secret1rzpet8pw8munm4qscke7ejxdgmu4k6zrph2la0",
                     "amount":"40000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"870",
                     "address":"secret1u49wzyu0cqln3mfajm608u54r8a4qkmac4xknl",
                     "amount":"60000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"81",
                     "address":"secret1j3l27ql3alzr2f4lq0pd4dqdv4795flvvf8sjm",
                     "amount":"40000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"264",
                     "address":"secret1gym8stav8lw35t72a7574cq0yrqlpkey9vvzfl",
                     "amount":"50000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"369",
                     "address":"secret1lhtz99434qg97lkdwnvqxvw24gq36cmtgt9gf4",
                     "amount":"120000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"86",
                     "address":"secret1e2h25sv8n7jacgvy62ktel6rnqsf2g2wlvxald",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"285",
                     "address":"secret1ugdtw4umlnmwa00vxj4t95h56ga2zk9l7t6v7d",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"424",
                     "address":"secret15yrchdg9npx23nk0d7ggzslmr28tsutxvc3qvy",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"373",
                     "address":"secret1vgxm9uskhx6a2q7h7vxq9zgtlglhegtfvtxks4",
                     "amount":"40000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"552",
                     "address":"secret17f33xs2regztfqsfasnwh63y93gzeqdxmfsrnh",
                     "amount":"10000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"726",
                     "address":"secret1ff3zt3w0tmexm76awrk9wepqq9dlhqeqhndzng",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"915",
                     "address":"secret1kune5zzmmmgl8l93getpxa6ydxtkj64hyesp0m",
                     "amount":"2000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"651",
                     "address":"secret1cw5qycynsnmgg32rkrjrnms8etw8v5tp56hpav",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"829",
                     "address":"secret17m3pk2788vgf8azsls4dl3g90p0lmf0d0qm848",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"935",
                     "address":"secret1jpyhjqlaq7gu06ndn3lwa6ys7wjkjuyqmkvffe",
                     "amount":"2000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"121",
                     "address":"secret1et77r0u62tjvqpnl4rammjj83vl7rql8hfc9m4",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"103",
                     "address":"secret1fv7m7s8dgxlgserp286xamt0rrrwvcgx480yu4",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"531",
                     "address":"secret1lut4ahy9mfawqjy8z8lnpfyzauh2unj4kjq9q7",
                     "amount":"40000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"717",
                     "address":"secret14rn2yd4wke7gfu7tga6l37k4lz2p8rh8chp07z",
                     "amount":"3750000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"896",
                     "address":"secret1p2mewruwwg0k8vf6cuj34n96lj8pcf78l2eu02",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"863",
                     "address":"secret1qqw29ytxyy335mxha9vql33w4h26aujqaap4qz",
                     "amount":"5000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"69",
                     "address":"secret1u7l308hqntjkfm5ja6yhd5zm2e6025zx60x7w2",
                     "amount":"6000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"922",
                     "address":"secret1duqps0fq32s2qwyqkxwguxvct6p7xyxy9lmyhg",
                     "amount":"3000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"123",
                     "address":"secret1sf88zztle5x4hput84xcjr4ns74srengt3374s",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"515",
                     "address":"secret1uhf5nlf66kw34h5uguefnzps6ld0zfs5vdde6m",
                     "amount":"12000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"174",
                     "address":"secret1n5uqcc6lpakx8q89xa8mjhm8cj798vwvwvej32",
                     "amount":"4000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"368",
                     "address":"secret1rvzsvd9m2ztlgw5vqv3zs58wc0a5gecpf92726",
                     "amount":"40000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"650",
                     "address":"secret1tw4eld6sg328tu7lz9f9jwdeewz9ekjpm7ljny",
                     "amount":"20000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"173",
                     "address":"secret1f7grq9rvn7tkdnggzfeyrkcgzyvhnnj0s479vh",
                     "amount":"85714000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"80",
                     "address":"secret1w08njtv0w2g269ar9hr4uayvhdvss3ch353fjq",
                     "amount":"114285000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"388",
                     "address":"secret1tpqxyxhfg9ay5ce25mq2l3atwywgc7jmrelhys",
                     "amount":"41666000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"317",
                     "address":"secret1gnhtrxv7wujh7lvdl7dunnxj2hsfa3nqty5q2e",
                     "amount":"5000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"286",
                     "address":"secret1mlu6zevn6kjkxc77fx229taknxegtwu9smrute",
                     "amount":"10000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"KOL1",
                     "address":"secret1mkkgwzvvtjltl70amuuycml4647lux449p9fvu",
                     "amount":"4000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"MCE",
                     "address":"secret1ced7z8uhz3vdj4vpp7fkc7pgmvz556csdaa0kk",
                     "amount":"95000000000000000000000",
                     "cliff":"0",
                     "start_at":7776000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"Founder7",
                     "address":"secret1h7sp840n0zq2ewpe4ucys60hjs67dwm8xwcljq",
                     "amount":"15000000000000000000000",
                     "cliff":"0",
                     "start_at":15552000,
                     "interval":86400,
                     "duration":51840000
                  }
               ]
            },
            {
               "partial":false,
               "name":"Public Sale",
               "total":"200000000000000000000000",
               "accounts":[
                  {
                     "name":"PublicSale",
                     "address":"secret1l9gwwsde2sd02z4jld649qqrp728wvxyerw046",
                     "amount":"200000000000000000000000",
                     "cliff":"200000000000000000000000",
                     "start_at":0,
                     "interval":0,
                     "duration":0
                  }
               ]
            },
            {
               "partial":false,
               "name":"Founders",
               "total":"2400000000000000000000000",
               "accounts":[
                  {
                     "name":"Founder1",
                     "address":"secret1qle5j097j99mn7xjpmdagsdd77zkz37ve7ww6g",
                     "amount":"790000000000000000000000",
                     "cliff":"0",
                     "start_at":15552000,
                     "interval":86400,
                     "duration":51840000
                  },
                  {
                     "name":"Founder2",
                     "address":"secret1ua8grtkqhezr2dad2vszqr9tqrlxftjlc8j70g",
                     "amount":"790000000000000000000000",
                     "cliff":"0",
                     "start_at":15552000,
                     "interval":86400,
                     "duration":51840000
                  },
                  {
                     "name":"Founder3",
                     "address":"secret17spd7qpjeje43q84kg507qtdafghj2mpq5j8xt",
                     "amount":"721000000000000000000000",
                     "cliff":"0",
                     "start_at":15552000,
                     "interval":86400,
                     "duration":51840000
                  },
                  {
                     "name":"Founder4",
                     "address":"secret1dsr05gju4xwn7n2mg2a0520khyuy40tn6r7kya",
                     "amount":"69000000000000000000000",
                     "cliff":"0",
                     "start_at":15552000,
                     "interval":86400,
                     "duration":51840000
                  },
                  {
                     "name":"Founder5",
                     "address":"secret1lvyxyuse0wp4wg4msh0s0ra9mul6m3f7js0dcn",
                     "amount":"15000000000000000000000",
                     "cliff":"0",
                     "start_at":15552000,
                     "interval":86400,
                     "duration":51840000
                  },
                  {
                     "name":"Founder6",
                     "address":"secret1yafvhyqmqz6dux07jgfw3mwwxf47r9gvj8glrp",
                     "amount":"15000000000000000000000",
                     "cliff":"0",
                     "start_at":15552000,
                     "interval":86400,
                     "duration":51840000
                  }
               ]
            },
            {
               "partial":false,
               "name":"DevFund",
               "total":"1300000000000000000000000",
               "accounts":[
                  {
                     "name":"DevN",
                     "address":"secret16avgwk5rqnlw6nsnw0mn2auw3xxjfhhewhdus0",
                     "amount":"1300000000000000000000000",
                     "cliff":"0",
                     "start_at":31536000,
                     "interval":2592000,
                     "duration":62208000
                  }
               ]
            },
            {
               "partial":true,
               "name":"Advisors",
               "total":"200000000000000000000000",
               "accounts":[
                  {
                     "name":"Advisor1",
                     "address":"secret1k6gkldusp2dawnfnrrxjjc9x3d7cynush7gutv",
                     "amount":"50000000000000000000000",
                     "cliff":"0",
                     "start_at":15552000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"Advisor2",
                     "address":"secret1uccknd9jmfufujlgzg9ce9r0gqlyjvjekpxuvc",
                     "amount":"50000000000000000000000",
                     "cliff":"0",
                     "start_at":15552000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"Advisor3",
                     "address":"secret1rj5hqd8n75yldu0dtp4u4kwepmh00q6392qwpq",
                     "amount":"10000000000000000000000",
                     "cliff":"0",
                     "start_at":15552000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"Advisor4",
                     "address":"secret12hzulrk78eddu5ugg7cj4nmrq5saq6nelmnzjr",
                     "amount":"5000000000000000000000",
                     "cliff":"0",
                     "start_at":15552000,
                     "interval":86400,
                     "duration":41472000
                  },
                  {
                     "name":"Advisor6",
                     "address":"secret1s64kjf7upgelkh825alyts67p4ytm79rpvrm5s",
                     "amount":"75000000000000000000000",
                     "cliff":"0",
                     "start_at":15552000,
                     "interval":86400,
                     "duration":41472000
                  }
               ]
            },
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

### Adding a new account to vesting

Send this message in a transaction from the admin address to the MGMT contract to unlock 1 SIENNA immediately for address `secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a`:

```json
{"add_account":{"pool":"Investors","account":{"name":"someone","amount":"1000000000000000000","address":"secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a","start_at":0,"interval":0,"duration":0,"cliff":"1000000000000000000"}}
```

>ℹ️ `start_at`, `interval`, `duration` are in seconds (1 day = 86400 seconds). For immediate vesting, set them all to 0 and `cliff` = `amount`. 

>ℹ️ `amount` and `cliff` are in attoSIENNA (multiply by `1000000000000000000` - 1 with 18 zeros - to get SIENNA), and must be in double quotes (`"`) - because JSON doesn't support numbers that big.

>⚠️ Be careful - errors here are permanent and can't be remedied without a full migration (untested procedure!)

Example:
```bash
# Sample transaction to add 1 SIENNA with immediate vesting to sample account
secretcli tx compute execute MGMT_CONTRACT_ADDRESS '{"add_account":{"pool_name":"Investors","account":{"name":"someone","amount":"1000000000000000000","address":"secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a","start_at":0,"interval":0,"duration":0,"cliff":"1000000000000000000"}}}' --from ADMIN_KEY_ALIAS --chain-id NETWORK_ID --gas 450000
```

> ℹ️ NETWORK_ID is holodeck-2 for testnet & secret-2 for mainnet
```
# Confirm the transaction succeeded
secretcli q compute tx TRANSACTION_HASH | jq '.'
```

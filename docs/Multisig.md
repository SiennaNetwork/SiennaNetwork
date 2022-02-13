# SIENNA - Multisig, deployment and setup

*Note: Because of the recently found issues with instantiating
multisig contracts via the official Secret Network CLI,
the setup below will outline how to deploy and instantiate the contracts
for the Sienna TGE from a regular (single) account,
and then transfer the ownership of those to a multisig account of choice,
which can later be used for execution of different admin actions.*

# Table of contents

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

## Create a multisig account

<table>

<tr><td valign="top">

### Step 1. Collect signers' public keys

In order to create the account first you need to collect
all of the relevant participating parties' public keys.

*Hint: You can also refer to the official secretcli documentation
as a helper [here](https://build.scrt.network/validators-and-full-nodes/secretcli.html#multisig-transactions).*

</td><td>

This example assumes you already have your own key in `secretcli`,
and are creating a 2-of-3 multisig with you and two other users.

```bash
secretcli keys add participant2 \
  --pubkey=secretpub1addwnpepqtd28uwa0yxtwal5223qqr5aqf5y57tc7kk7z8qd4zplrdlk5ez5kdnlrj4
secretcli keys add participant3 \
  --pubkey=secretpub1addwnpepqgj04jpm9wrdml5qnss9kjxkmxzywuklnkj0g3a3f8l5wx9z4ennz84ym5t
```

</td></tr>

<tr><!--spacer--></tr>

<tr><td valign="top">

### Step 2. Create multisig wallet

Use the aliases of the imported accounts to form a new multisig account.

Set up the desired threshold of required signatures for a valid transaction.

</td><td>

**Note:** `--multisig-threshold=2` means 2 of the 3 signatures in the multisig
are enough to execute a valid transaction.

```bash
# create the multisig account with alias `MULTISIG_ACCOUNT`
secretcli keys add     \
  MULTISIG_ACCOUNT      \
  --multisig-threshold=2 \
  --multisig=MY_ACCOUNT_ALIAS,participant2,participant3
```

To confirm the above command worked:

```bash
secretcli keys show MULTISIG_ACCOUNT -a
# As output you should see the multisig account address
```

</td></tr>

</table>

## Performing multisig transactions

<table>
<tr><td valign="top">

### Step 1. Get the required certificates

These certificates are needed to encrypt the tx data with the node's secure enclave.

</td><td>

```bash
secretcli query register secret-network-params
```

*The command above will download the needed certificates in the current directory,
make sure you proceed further within the same directory or address the path to the
certificate properly in the next steps.*

</td></tr>

<tr><!--spacer--></tr>

<tr><td>

### 2. Prepare offline transaction for the multisig account

In the example below we will demonstrate how to prepare a multisig transaction
that executes a smart contract method. For example minting tokens from a SNIP20
contract via the `mint` method, assuming the contract is owned by the multisig
and the multisig account has minter access granted.

</td><td>

The command above will give you the information about `MULTISIG_ACCOUNT_SEQUENCE` and `MULTISIG_ACCOUNT_NUMBER` that you will need in order to proceed further and succesfully create an offline multisig tx.

```bash
# Get multisig sequence and account number
secretcli q account MULTISIG_ACCOUNT_ADDRESS

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

</td></tr>

<tr><!--spacer--></tr>

<tr><td>

Continue with preparation of the transaction towards the `SMART_CONTRACT_ADDRESS`, containing the message that we want to send.

</td><td>

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

</td></tr>

<tr><!--spacer--></tr>

<tr><td>

### 3. Distribute for signatures

Now that we have the prepared transaction its time to collect the signatures required.
Depending on the threshold that was set for the multisig - you can distribute the file
`unsignedTx.tx` to the required amount of signers by any means of communication
(IM, email, etc.).

</td><td>

### 4. Individual signing

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

</td></tr>

<tr><!--spacer--></tr>

<tr><td>

### 5. Preparing signed transaction using the collected signatures

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

</td><td>

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
          "msg": "...LONG BASE64-ENCODED CIPHERTEXT...",
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

</td></tr>

<tr><!--spacer--></tr>

<tr><td>

### 6. Broadcasting the tx

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

</td><td>

If all went well you should see similar output:

```json=
{
  "type": "execute",
  "raw_input": "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084{\"mint\":{\"recipient\":\"secret14rn2yd4wke7gfu7tga6l37k4lz2p8rh8chp07z\", \"amount\": \"20000000000000000000\" }}",
  "input": null,
  "output_data": "...LONG BASE64-ENCODED CIPHERTEXT...",
  "output_data_as_string": "{\"mint\":{\"status\":\"success\"}}                                                                                                                                                                                                                                   ",
  "output_log": [],
  "output_error": {},
  "plaintext_error": ""
}
```

</td></tr>

<tr><!--spacer--></tr>

<tr><td></td><td></td></tr>

<tr><!--spacer--></tr>

</table>

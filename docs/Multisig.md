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

### Step 1. Get the required certificates for the chain

These certificates are needed to encrypt the transaction data for the node's secure enclave.

Run the following command:

```bash
secretcli query register secret-network-params
```

> This will download the certificates in the current directory.
> Make sure you don't leave it for the next steps -
> or modify them to point to the certificate path.

</td><td>

```jsonc
// TODO: example of what the certificates look like
```

</td></tr>

<tr><!--spacer--></tr>

<tr><td valign="top">

### Step 2. Get the account number and sequence for the multisig account

You need the `account_number` and `sequence` of the multisig account to create
an unsigned transaction file.
  * The `account_number` remains constant for the address.
  * The `sequence` increments with every transaction from that address to prevent replay attacks.

Now is a good time to set the address of the multisig account in your shell session,
so you don't have to manually type it into the following commands.

```bash
export MULTISIG='secret1...' # see "Create a multisig account"
```

Get the account number and sequence:

```bash
secretcli q account "$MULTISIG"
```

Now is a good time to set the account number and sequence in your shell session:

```bash
export ACCOUNT='...'  # values from `secretcli q account`
export SEQUENCE='...'
```

</td><td>

Example output of `secretcli q account`:

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

<tr><td valign="top">

### Step 3. Create the unsigned transaction file

This file contains the messages that you want to send,
encrypted with the smart contract's code hash.

Now is a good time to set the address and code hash of the contract
that you will be calling in your shell session:

```bash
export CONTRACT='secret1...' # this should be the address of the contract
```

To create an unsigned transaction file containing one
`execute` call:

```bash
export UNSIGNED='unsigned.tx.json' # name of file in which to store the unsigned transaction
export CONTRACT='secret1...'       # the address of the contract that you'll be calling
export CODE_HASH='...'             # the code hash of the contract that you'll be calling
export MESSAGE='{"mint": {"recipient": "secret1...", "amount": "10000" }}' # the message to execute
secretcli tx compute execute "$CONTRACT" "$MESSAGE" \
  --code-hash "$CODE_HASH"   \
  --gas 450000                \
  --chain-id secret-4          \
  --from "$MULTISIG"            \
  --sequence "$SEQUENCE"         \
  --account-number "$ACCOUNT"     \
  --enclave-key io-master-cert.der \
  --generate-only > unsigned.tx.json
```

</td><td>

Example contents of `unsigned.tx.json`:

```json=
{
  "type": "cosmos-sdk/StdTx",
  "value": {
    "msg": [
      {
        "type": "wasm/MsgExecuteContract",
        "value": {
          "sender": "secret1...",
          "contract": "secret1...",
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
    "signatures": null,
    "memo": ""
  }
}
```

</td></tr>

<tr><!--spacer--></tr>

<tr><td>

### Step 4. Collect signatures

Now that we have the prepared transaction, it's time to collect the signatures required.

Run the following command to generate the actual signing command:

```bash
secretcli tx sign "$UNSIGNED" \
--offline               \
--multisig="$MULTISIG"   \
--chain-id=secret-4       \
--from="$MULTISIG"         \
--account-number="$ACCOUNT" \
--sequence="$SEQUENCE"       \
--output-document=signed_N.tx.json
```

* Distribute the file `unsigned.tx.json`, and the generated signing command,
to as many signers as needed to achieve the threshold originally defined for the multisig.
You can use standard communication channels such as text messaging, email, etc.

* Instruct the signers to sign the transaction by running the generated command in the
directory in which they downloaded the unsigned transaction file.

* Collect the individual `signed_N.tx.json` files collected from the signers.

</td><td>

>ℹ️ name the different files with the number of the signer for conveinience.

Example file output:

```json=
{
  "pub_key": {
    "type": "tendermint/PubKeySecp256k1",
    "value": "AusMnK+0hQCDvZZ8QPNXazpA2NCy1/WoTlgUMyVOJZxI"
  },
  "signature": "...LONG BASE64-ENCODED SIGNATURE..."
}
```

</td></tr>

<tr><!--spacer--></tr>

<tr><td>

### 5. Preparing signed transaction using the collected signatures

After the signatures has been collected you should have a set of multiple files containing signatures for the accounts participating in the multisig and the `unsigned.tx.json` file that contains the transaction information.

```bash
# Multisign the transaction with the collected signatures
secretcli tx multisign unsigned.tx.json \
 MULTISIG_ALIAS \
 mint_signed_participant1.json \
 mint_signed_participant2.json \
 --offline \
 --account-number=MULTISIG_ACCOUNT_NUMBER \
 --sequence=MULTISIG_ACCOUNT_SEQUENCE \
 > signed.tx.json
```

</td><td>

Example output content of the `signed.tx.json` file:
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
        "signature": "...LONG BASE64-ENCODED SIGNATURE..."
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
secretcli tx broadcast signed.tx.json
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
  "raw_input": "...HEXADECIMAL CODE HASH AND PLAINTEXT MESSAGE...",
  "input": null,
  "output_data": "...LONG BASE64-ENCODED CIPHERTEXT...",
  "output_data_as_string": "{\"mint\":{\"status\":\"success\"}},
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

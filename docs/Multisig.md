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

A multisig account is used for collective ownership of the administrative access
over the platform's core smart contracts (MGMT, RPT, AMM Factory, Rewards).

Here's how to create one.

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

So you have the admin of the contract set to a multisig account.
How to collectively administrate the contract then?

Send transactions signed by at least N of the owners of the multisig,
where N is the value of `--multisig-threshold` used when creating the
multisig.

Here's how to send a single multisig transaction using just `secretcli`.
To send a bundle of transactions from the multisig account, use Fadroma.

<table>

<tr valign="top"><td>

### Step 1. Define the transaction that you will be signing

Open a new Bash shell and define the following environment variables:

```bash
export CHAIN='secret-4'
export MULTISIG='secret1...'       
export CONTRACT='secret1...'       
export UNSIGNED='UnsignedTX.json' 
export CODE_HASH='...'             
export MESSAGE='...'
```

</td><td>

**Output environment of step 1:**

* `CHAIN`     - ID of executing chain
* `MULTISIG`  - address or name of multisig from "Create a multisig account"
* `CONTRACT`  - the address of the contract that you'll be calling
* `UNSIGNED`  - name of file in which to store the unsigned transaction
* `CODE_HASH` - the code hash of the contract that you'll be calling
* `MESSAGE`   - the message to execute

**Example value for `MESSAGE`:**

```json=
{"mint":{"recipient":"some-address","amount":"10000"}}
```

</td></tr>

<tr><!--spacer--></tr>

<tr><td valign="top">

### Step 2. Populate the signing environment

Download the chain's global certificates to the current directory
with this command:

```bash
secretcli query register secret-network-params
```

> Make sure to stay in the same directory throughout.

Get the account number and chain ID for the multisig with this command:

```bash
secretcli q account "$MULTISIG"
```

Export them to your Bash shell session with:

```bash
export ACCOUNT='...'  # from `secretcli q account`
export SEQUENCE='...' # from `secretcli q account`
```

You need these to create an unsigned transaction file.
* The `account_number` remains constant for the address.
* The `sequence` increments with every transaction from that address to prevent replay attacks.

</td><td>

**Output files of step 2:**

```json=
// io-master-cert.der
```

**Output environment of step 2:**

* `ACCOUNT`
* `SEQUENCE`

**Example output of `secretcli q account`:**

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

This file will contain the messages that you want to send,
encrypted with the smart contract's code hash.

To create an unsigned transaction file containing one
`execute`, run this command:

```bash
secretcli tx compute execute  \
  "$CONTRACT"                  \
  "$MESSAGE"                    \
  --code-hash       "$CODE_HASH" \
  --gas             450000        \
  --chain-id        "$CHAIN"       \
  --from            "$MULTISIG"     \
  --sequence        "$SEQUENCE"      \
  --account-number  "$ACCOUNT"        \
  --enclave-key     io-master-cert.der \
  --generate-only > "$UNSIGNED"
```

>This will create a file named after the value of the
>`UNSIGNED` environment variable. E.g. if `UNSIGNED=foo.json`
>it will write the unsigned transaction to `foo.json` in the
>current directory.

</td><td>

**Output files of step 3:**

- `$UNSIGNED` Unsigned transaction file (e.g. `UnsignedTX.json`)

**Example contents of unsigned transaction:**

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

Now that we have prepared the unsigned transaction,
it's time to collect the signatures required.

Run the following command to generate an instruction file
containing the actual signing command:

```bash
envsubst << EOF > INSTRUCTIONS.txt

Your signature has been requested on
the transaction contained in "$UNSIGNED".

Run the following command to sign:

  secretcli tx sign "$UNSIGNED" \
  --offline               \
  --multisig="$MULTISIG"   \
  --chain-id=secret-4       \
  --from="$MULTISIG"         \
  --account-number="$ACCOUNT" \
  --sequence="$SEQUENCE"       \
  --output-document=Signature.json

Or use Motika (https://github.com/hackbg/motika)
for a graphical signing experience.

EOF
```

* Distribute the **unsigned transaction file** and the **signing instructions**,
  to as many signers as needed to achieve the threshold originally defined for the multisig.

>You can use standard communication channels such as text messaging, email, etc.
>The contents of the unsigned transaction are not a cryptographic secret.

* Collect the signatures that the signers have generated using the signing instructions.

</td><td>

**Output files of step 4:**
- `Signature_1.json`
- `Signature_2.json`
- ...
- `Signature_N.json`

**Example contents of signature file:**

```json=
{
  "pub_key": {
    "type": "tendermint/PubKeySecp256k1",
    "value": "BASE64-ENCODED PUBLIC KEY"
  },
  "signature": "BASE64-ENCODED SIGNATURE"
}
```

</td></tr>

<tr><!--spacer--></tr>

<tr><td>

### Step 5. Prepare the multisigned transaction using the collected signatures

After the signatures has been collected,
you should have a set of multiple files containing signatures
for the accounts participating in the multisig, and the unsigned transaction file
that contains the transaction information.

Run the following commant to multisign the transaction with the collected signatures:

```bash
secretcli tx multisign  \
  "$UNSIGNED"            \
  "$MULTISIG"             \
  "Signature_1.json"       \
  "Signature_2.json"        \ # repeat for all signatures
  --offline                  \
  --account-number="$ACCOUNT" \
  --sequence="$SEQUENCE" > Signed.json
```

</td><td>

**Output files of step 5:**
- `Signed.json` - signed transaction

**Example contents of signed transaction file:**
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
          "msg": "BASE64-ENCODED CIPHERTEXT",
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
                "value": "BASE64-ENCODED PUBKEY"
              },
              {
                "type": "tendermint/PubKeySecp256k1",
                "value": "BASE64-ENCODED PUBKEY"
              },
              {
                "type": "tendermint/PubKeySecp256k1",
                "value": "BASE64-ENCODED PUBKEY"
              }
            ]
          }
        },
        "signature": "BASE64-ENCODED MULTI-SIGNATURE"
      }
    ],
    "memo": ""
  }
}
```

</td></tr>

<tr><!--spacer--></tr>

<tr><td>

### Step 6. Broadcast the multisigned transaction

Now all that is left is to broadcast the transaction to the network.

```bash
secretcli tx broadcast signed.tx.json
```

As a response you will see the transaction hash similar to this:

Then you can verify the transaction was executed succesfully via the following:

```bash
secretcli q compute tx TX_HASH
```

</td><td>

**Example output of `secretcli tx broadcast`:**

```json=
{"height":"0","txhash":"BASE16-ENCODED TRANSACTION HASH","raw_log":"[]"}
```

**Example output of `secretcli q compute tx`:**

```json=
{
  "type": "execute",
  "raw_input": "BASE16-ENCODED CODE HASH AND PLAINTEXT MESSAGE",
  "input": null,
  "output_data": "BASE64-ENCODED CIPHERTEXT",
  "output_data_as_string": "JSON-ENCODED OUTPUT OF TRANSACTION",
  "output_log": [],
  "output_error": {},
  "plaintext_error": ""
}
```

</td></tr>

<tr><!--spacer--></tr>

</table>

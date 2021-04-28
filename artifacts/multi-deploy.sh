#!/usr/bin/env bash
set -aeux
set -o pipefail

BLOB=snip20-reference-impl@HEAD.wasm
SELECTED_NETWORK=enigma-pub-testnet-3 # use holodeck-2 for testnet and secret-2 for mainnet
CHAIN_ID="--chain-id $SELECTED_NETWORK"
UPLOAD_GAS="--gas 1600000"
WAIT_OK="-b block --yes"
UPLOADER="a"
UPLOAD_TX=`secretcli tx compute store --from "$UPLOADER" "$BLOB" $CHAIN_ID $UPLOAD_GAS $WAIT_OK`
UPLOAD_TX_HASH=`echo "$UPLOAD_TX" | jq -r .txhash`
UPLOAD_TX_INFO=`secretcli q tx $UPLOAD_TX_HASH`
CODE_ID=`echo "$UPLOAD_TX_INFO" | jq -r .logs[0].events[0].attributes[3].value`
CODE_HASH=0f64ab3bab7f1d0194e9e2dd3a69108d323c9b52363ef55d4bba44e4f0d48462

UNSIGNED=unsigned.json
PRNG_SEED=`cat /dev/urandom | env LC_CTYPE=C tr -dc 'a-zA-Z0-9' | fold -w 32 | head -n 1`
INIT_MSG="{\"prng_seed\": \"$PRNG_SEED\", \"name\": \"Sienna\", \"symbol\": \"SIENNA\", \"decimals\": 18, \"config\": { \"public_total_supply\": true } }"
DATE=`date -u +"%Y-%m-%dT%H:%M:%SZ"`

MULTISIG_NAME=smt1
MULTISIG_ADDRESS=`secretcli keys show -a $MULTISIG_NAME`
MULTISIG_ACC=`secretcli q account "$MULTISIG_ADDRESS" | jq -r .value.account_number`
MULTISIG_SEQ=`secretcli q account "$MULTISIG_ADDRESS" | jq -r .value.sequence`
secretcli tx compute instantiate $CODE_ID "$INIT_MSG" --label "SIENNA SNIP20 ($DATE)" \
  --from "$MULTISIG_ADDRESS" --chain-id "$SELECTED_NETWORK" --enclave-key io-master-cert.der \
  --code-hash "$CODE_HASH" --generate-only | tee "$UNSIGNED"

SIGNER_1_NAME=t1
SIGNER_1_ADDRESS=`secretcli keys show -a $SIGNER_1_NAME`
SIGNER_1_ACC=`secretcli q account "$SIGNER_1_ADDRESS" | jq -r .value.account_number`
SIGNER_1_SEQ=`secretcli q account "$SIGNER_1_ADDRESS" | jq -r .value.sequence`
SIGNATURE_1=p1.json
secretcli tx sign "$UNSIGNED" --multisig "$MULTISIG_ADDRESS" --from "$SIGNER_1_NAME"\
  --offline $CHAIN_ID --account-number="$MULTISIG_ACC" --sequence="$MULTISIG_SEQ" \
  --output-document "$SIGNATURE_1"

SIGNER_2_NAME=t2
SIGNER_2_ADDRESS=`secretcli keys show -a $SIGNER_2_NAME`
SIGNER_2_ACC=`secretcli q account "$SIGNER_2_ADDRESS" | jq -r .value.account_number`
SIGNER_2_SEQ=`secretcli q account "$SIGNER_2_ADDRESS" | jq -r .value.sequence`
SIGNATURE_2=p2.json
secretcli tx sign "$UNSIGNED" --multisig=$MULTISIG_ADDRESS --from "$SIGNER_2_NAME" \
  --offline $CHAIN_ID --account-number="$MULTISIG_ACC" --sequence="$MULTISIG_SEQ" \
  --output-document "$SIGNATURE_2"

SIGNED=signed.json
secretcli tx multisign "$UNSIGNED" "$MULTISIG_NAME" "$SIGNATURE_1" "$SIGNATURE_2" \
  --offline $CHAIN_ID --account-number="$MULTISIG_ACC" --sequence="$MULTISIG_SEQ" \
  | tee "$SIGNED"

INIT_TX=`secretcli tx broadcast "$SIGNED" $WAIT_OK`
INIT_TX_HASH=`echo $INIT_TX | jq -r .txhash`
# CONTRACT_ADDRESS=`secretcli q tx $INIT_TX_HASH | jq -r .logs[0].events[0].attributes[4].value`

CONTRACT_ADDRESS=`secretcli q tx $INIT_TX_HASH | jq .`
echo "SIENNA token is now live at $CONTRACT_ADDRESS"

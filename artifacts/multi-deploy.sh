#!/usr/bin/env bash
set -xaue

SECRETCLI="./secretcli-linux-amd64"
$SECRETCLI query register secret-network-params

BLOB=snip20-reference-impl@HEAD.wasm
SELECTED_NETWORK=holodeck-2 #enigma-pub-testnet-3 # use holodeck-2 for testnet and secret-2 for mainnet
CHAIN_ID="--chain-id $SELECTED_NETWORK"
UPLOAD_GAS="--gas 1600000"
WAIT_OK="-b block --yes"
UPLOADER="t1"
UPLOAD_TX=`$SECRETCLI tx compute store --from "$UPLOADER" "$BLOB" $CHAIN_ID $UPLOAD_GAS $WAIT_OK`
UPLOAD_TX_HASH=`echo "$UPLOAD_TX" | jq -r .txhash`
UPLOAD_TX_INFO=`$SECRETCLI q tx $UPLOAD_TX_HASH`
CODE_ID=`echo "$UPLOAD_TX_INFO" | jq -r .logs[0].events[0].attributes[3].value`
CODE_HASH=c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084

UNSIGNED=unsigned.json
PRNG_SEED=`cat /dev/urandom | env LC_CTYPE=C tr -dc 'a-zA-Z0-9' | fold -w 32 | head -n 1`
INIT_MSG="{\"prng_seed\": \"$PRNG_SEED\", \"name\": \"Sienna\", \"symbol\": \"SIENNA\", \"decimals\": 18, \"config\": { \"public_total_supply\": true } }"
DATE=`date -u +"%Y-%m-%dT%H:%M:%SZ"`

MULTISIG_NAME=smt1
MULTISIG_ADDRESS=`$SECRETCLI keys show -a $MULTISIG_NAME`
MULTISIG_ACC=`$SECRETCLI q account "$MULTISIG_ADDRESS" | jq -r .value.account_number`
MULTISIG_SEQ=`$SECRETCLI q account "$MULTISIG_ADDRESS" | jq -r .value.sequence`
$SECRETCLI tx compute instantiate $CODE_ID "$INIT_MSG" --label "SIENNA SNIP20 ($DATE)" \
  --from "$MULTISIG_ADDRESS" --chain-id "$SELECTED_NETWORK" --enclave-key io-master-cert.der \
  --code-hash "$CODE_HASH" --generate-only | tee "$UNSIGNED"

SIGNER_1_NAME=t1
SIGNER_1_ADDRESS=`$SECRETCLI keys show -a $SIGNER_1_NAME`
SIGNER_1_ACC=`$SECRETCLI q account "$SIGNER_1_ADDRESS" | jq -r .value.account_number`
SIGNER_1_SEQ=`$SECRETCLI q account "$SIGNER_1_ADDRESS" | jq -r .value.sequence`
SIGNATURE_1=p1.json
$SECRETCLI tx sign "$UNSIGNED" --multisig "$MULTISIG_ADDRESS" --from "$SIGNER_1_NAME"\
  --offline $CHAIN_ID --account-number="$MULTISIG_ACC" --sequence="$MULTISIG_SEQ" \
  --output-document "$SIGNATURE_1"

SIGNER_2_NAME=t2
SIGNER_2_ADDRESS=`$SECRETCLI keys show -a $SIGNER_2_NAME`
SIGNER_2_ACC=`$SECRETCLI q account "$SIGNER_2_ADDRESS" | jq -r .value.account_number`
SIGNER_2_SEQ=`$SECRETCLI q account "$SIGNER_2_ADDRESS" | jq -r .value.sequence`
SIGNATURE_2=p2.json
$SECRETCLI tx sign "$UNSIGNED" --multisig=$MULTISIG_ADDRESS --from "$SIGNER_2_NAME" \
  --offline $CHAIN_ID --account-number="$MULTISIG_ACC" --sequence="$MULTISIG_SEQ" \
  --output-document "$SIGNATURE_2"

SIGNED=signed.json
$SECRETCLI tx multisign "$UNSIGNED" "$MULTISIG_NAME" "$SIGNATURE_1" "$SIGNATURE_2" \
  --offline $CHAIN_ID --account-number="$MULTISIG_ACC" --sequence="$MULTISIG_SEQ" \
  | tee "$SIGNED"

INIT_TX=`$SECRETCLI tx broadcast "$SIGNED" $WAIT_OK`
INIT_TX_HASH=`echo $INIT_TX | jq -r .txhash`
# CONTRACT_ADDRESS=`$SECRETCLI q tx $INIT_TX_HASH | jq -r .logs[0].events[0].attributes[4].value`

CONTRACT_ADDRESS=`$SECRETCLI q tx $INIT_TX_HASH | jq .`
echo "SIENNA token is now live at $CONTRACT_ADDRESS"

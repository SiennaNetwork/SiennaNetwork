#!/usr/bin/env bash
set -aemu
export MY_ADDRESS=secret1pv6kxaqnn7nqyxlmyr47hqjlekc88njxhfzqnr
UPLOAD_TX_HASH=`secretcli tx compute store snip20-reference-impl@HEAD.wasm --from "$MY_ADDRESS" --gas 1600000 -b block --yes | jq -r .txhash`
CODE_ID=`secretcli q tx $UPLOAD_TX_HASH | jq -r .logs[0].events[0].attributes[3].value`
PRNG_SEED=`cat /dev/urandom | env LC_CTYPE=C tr -dc 'a-zA-Z0-9' | fold -w 32 | head -n 1`
INIT_MSG="{\"prng_seed\": \"$PRNG_SEED\", \"name\": \"Sienna\", \"symbol\": \"SIENNA\", \"decimals\": 18, \"config\": { \"public_total_supply\": true } }"
DATE=`date -u +"%Y-%m-%dT%H:%M:%SZ"`
INIT_TX_HASH=`secretcli tx compute instantiate $CODE_ID "$INIT_MSG" --label "SIENNA SNIP20 ($DATE)" --from "$MY_ADDRESS" -b block --yes | jq -r .txhash`
CONTRACT_ADDRESS=`secretcli q tx $INIT_TX_HASH | jq -r .logs[0].events[0].attributes[4].value`
echo "SIENNA token is now live at $CONTRACT_ADDRESS"

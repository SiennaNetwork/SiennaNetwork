#!/bin/bash

set -e

function wait_for_tx() {
  until (secretcli q tx "$1"); do
      sleep 5
  done
}

export wasm_path=build

export deployer_name=sefi
export deployer_address=$(secretcli keys show -a $deployer_name)
#export viewing_key="api_key_0ddf91b8f7a96314e01ff3ebe6fb20cf"
export prng_seed="gXBwdMGcNVbC8CtqptkwZeBwRScR7m5+LNbTrxoTHx4hLajYhrjIkmpaFLFXKStJTUT6mMS6wLGcUAkYSCPyfQ=="
echo "Deployer address: '$deployer_address'"
#echo "Viewing key: '$viewing_key'"

echo "Storing SEFI"
secretcli tx compute store "${wasm_path}/gov_token.wasm" --from "$deployer_name" --gas 3000000 -b block -y
token_code_id=$(secretcli query compute list-code | jq -r '.[-1]."id"')
echo "Stored SEFI: '$token_code_id'"

echo "Deploying Gov Token.."
export TX_HASH=$(
  secretcli tx compute instantiate $token_code_id '{"admin": "'$deployer_address'", "symbol": "SEFI", "decimals": 6, "initial_balances": [{"address":"secret1f2mf5xusm28a2pvzu5ztu58c5w89kdqjcy4sfw","amount":"2631197000000"},{"address":"secret1u6e6ps5j8zgg0skt5ygseg5n39hmngxq2tr2ep","amount":"1429999000000"},{"address":"secret1v5tewc3c5gk98z8uv6m295fgpcr8ulr2kale2r","amount":"2443032000000"}], "prng_seed": "'"$prng_seed"'", "name": "Secret Finance", "config":{"public_total_supply": true}}' --from $deployer_name --gas 1500000 --label SEFI -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH
sefi_addr=$(secretcli query compute list-contract-by-code $token_code_id | jq -r '.[-1].address')
echo "SEFI address: '$sefi_addr'"

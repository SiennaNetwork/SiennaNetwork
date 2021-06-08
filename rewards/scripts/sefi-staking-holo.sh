#!/bin/bash

set -e

function wait_for_tx() {
  until (secretcli q tx "$1"); do
      sleep 5
  done
}

export wasm_path=build

export revision="12"
export deployer_name=holotest
export viewing_key="123"
export gov_addr="secret12q2c5s5we5zn9pq43l0rlsygtql6646my0sqfm"
export token_code_hash="c7fe67b243dfedc625a28ada303434d6f5a46a3086e7d2b5063a814e9f9a379d"
export master_addr="secret13hqxweum28nj0c53nnvrpd23ygguhteqggf852"
export master_code_hash="c8555c2de49967ca484ba21cf563c2b27227a39ad6f32ff3de9758f20159d2d2"

echo "Storing SEFI Staking"
resp=$(secretcli tx compute store "${wasm_path}/lp_staking.wasm" --from "$deployer_name" --gas 3000000 -b block -y)
echo $resp
sefi_staking_code_id=$(echo $resp | jq -r '.logs[0].events[0].attributes[] | select(.key == "code_id") | .value')
echo "Stored lp staking: '$sefi_staking_code_id', '$sefi_staking_hash'"

echo "Deploying SEFI Staking Contract.."
export TX_HASH=$(
  secretcli tx compute instantiate $sefi_staking_code_id '{"reward_token":{"address":"'"$gov_addr"'", "contract_hash":"'"$token_code_hash"'"},"inc_token":{"address":"'"$gov_addr"'", "contract_hash":"'"$token_code_hash"'"},"master":{"address":"'"$master_addr"'", "contract_hash":"'"$master_code_hash"'"},"viewing_key":"'"$viewing_key"'","token_info":{"name":"sefis","symbol":"SEFISTAKING"},"prng_seed":"YWE="}' --from $deployer_name --gas 1500000 --label sefi-stake-$revision -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH
sefi_staking_addr=$(secretcli query compute list-contract-by-code $sefi_staking_code_id | jq -r '.[-1].address')

sefi_staking_hash="$(secretcli q compute contract-hash "$sefi_staking_addr")"
sefi_staking_hash="${sefi_staking_hash:2}"

echo "Setting SEFI Staking weight.."
export TX_HASH=$(
  secretcli tx compute execute "$master_addr" '{"set_weights":{"weights":[{"address":"'"$sefi_staking_addr"'","hash":"'"$sefi_staking_hash"'","weight":99}]}}' --from $deployer_name --gas 1500000 -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH

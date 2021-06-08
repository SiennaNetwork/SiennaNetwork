#!/bin/bash

set -e

function secretcli() {
  export docker_name=secretdev
  docker exec "$docker_name" secretcli "$@";
}

function wait_for_tx() {
  until (secretcli q tx "$1"); do
      sleep 5
  done
}

export wasm_path=/root/code/build

export deployer_name=a
export deployer_address=$(secretcli keys show -a $deployer_name)
echo "Deployer address: '$deployer_address'"
export viewing_key="123"
echo "Viewing key: '$viewing_key'"

secretcli tx compute store "${wasm_path}/gov_token.wasm" --from "$deployer_name" --gas 3000000 -b block -y
token_code_id=$(secretcli query compute list-code | jq -r '.[-1]."id"')
token_code_hash=$(secretcli query compute list-code | jq -r '.[-1]."data_hash"')
echo "Stored token: '$token_code_id', '$token_code_hash'"

secretcli tx compute store "${wasm_path}/weight_master.wasm" --from "$deployer_name" --gas 3000000 -b block -y
master_code_id=$(secretcli query compute list-code | jq -r '.[-1]."id"')
master_code_hash=$(secretcli query compute list-code | jq -r '.[-1]."data_hash"')
echo "Stored master: '$master_code_id'"

secretcli tx compute store "${wasm_path}/lp_staking.wasm" --from "$deployer_name" --gas 3000000 -b block -y
lp_staking_code_id=$(secretcli query compute list-code | jq -r '.[-1]."id"')
lp_staking_hash=$(secretcli query compute list-code | jq -r '.[-1]."data_hash"')
echo "Stored lp staking: '$lp_staking_code_id', '$lp_staking_hash'"

echo "Deploying Gov Token.."
export TX_HASH=$(
  secretcli tx compute instantiate $token_code_id '{"admin": "'$deployer_address'", "symbol": "SEFI", "decimals": 6, "initial_balances": [], "prng_seed": "YWE=", "name": "SEFI"}' --from $deployer_name --gas 1500000 --label SEFI -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH
gov_addr=$(secretcli query compute list-contract-by-code $token_code_id | jq -r '.[-1].address')
echo "SEFI address: '$gov_addr'"

echo "Deploying Master Contract.."
export TX_HASH=$(
  secretcli tx compute instantiate $master_code_id '{"gov_token_addr":"'"$gov_addr"'","gov_token_hash":"'"$token_code_hash"'","minting_schedule":[{"end_block":1000000,"mint_per_block":"1000000000"}]}' --from $deployer_name --gas 1500000 --label MASTER -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH
master_addr=$(secretcli query compute list-contract-by-code $master_code_id | jq -r '.[-1].address')
echo "Master address: '$master_addr'"

echo "Deploying LP Token.."
export TX_HASH=$(
  secretcli tx compute instantiate $token_code_id '{"admin": "'$deployer_address'", "symbol": "LPT", "decimals": 6, "initial_balances": [{"address": "'$deployer_address'", "amount": "100000000000000000000000"}], "prng_seed": "YWE=", "name": "LPT"}' --from $deployer_name --gas 1500000 --label LP -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH
lp_token_addr=$(secretcli query compute list-contract-by-code $token_code_id | jq -r '.[-1].address')
echo "LP Token address: '$lp_token_addr'"

echo "Deploying LP Staking Contract.."
export TX_HASH=$(
  secretcli tx compute instantiate $lp_staking_code_id '{"reward_token":{"address":"'"$gov_addr"'", "contract_hash":"'"$token_code_hash"'"},"inc_token":{"address":"'"$lp_token_addr"'", "contract_hash":"'"$token_code_hash"'"},"master":{"address":"'"$master_addr"'", "contract_hash":"'"$master_code_hash"'"},"viewing_key":"'"$viewing_key"'","token_info":{"name":"lps","symbol":"LPS"},"prng_seed":"YWE="}' --from $deployer_name --gas 1500000 --label LPSTAKING -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH
lp_staking_addr=$(secretcli query compute list-contract-by-code $lp_staking_code_id | jq -r '.[-1].address')
echo "LP Staking address: '$lp_staking_addr'"
echo "LP Token address: '$lp_token_addr'"
echo "Master address: '$master_addr'"
echo "SEFI address: '$gov_addr'"

echo "Setting Master as a SEFI minter and discarding admin as a minter"
export TX_HASH=$(
  secretcli tx compute execute "$gov_addr" '{"set_minters":{"minters":["'"$master_addr"'"]}}' --from $deployer_name --gas 1500000 -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH

echo "Setting LP Staking weight.."
export TX_HASH=$(
  secretcli tx compute execute "$master_addr" '{"set_weights":{"weights":[{"address":"'"$lp_staking_addr"'","hash":"'"$lp_staking_hash"'","weight":5}]}}' --from $deployer_name --gas 1500000 -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH

echo "Sleeping 10 sec.."
sleep 10

echo "Staking LP Token.."
deposit_msg=$(base64 <<< '{"deposit":{}}')
export TX_HASH=$(
  secretcli tx compute execute "$lp_token_addr" '{"send":{"recipient":"'"$lp_staking_addr"'","amount":"1000","msg":"'"$deposit_msg"'"}}' --from $deployer_name --gas 1500000 -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH

echo "Setting viewing keys.."
secretcli tx snip20 set-viewing-key $gov_addr "123" -b block --from $deployer_name -y
secretcli tx snip20 set-viewing-key $lp_token_addr "123" -b block --from $deployer_name -y
secretcli tx snip20 set-viewing-key $lp_staking_addr "123" -b block --from $deployer_name -y

echo "a address: $deployer_address"

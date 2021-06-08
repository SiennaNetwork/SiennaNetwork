#!/bin/bash

set -e

function wait_for_tx() {
  until (secretcli q tx "$1"); do
      sleep 5
  done
}

export wasm_path=build

export revision="10"
export deployer_name=holotest
export deployer_address=$(secretcli keys show -a $deployer_name)
echo "Deployer address: '$deployer_address'"
export viewing_key="123"
echo "Viewing key: '$viewing_key'"

export lp_token1="secret1qjr5rsuz45mpx6hrwud5fsw25039y0zgr4eu4x" # sSCRT<->sETH
export lp_token2="secret1c0rt9zj8efmr8w5hf2w0stwahgfur6j7y7x4pn" # sSCRT<->SCRT
export lp_token_hash="ea3df9d5e17246e4ef2f2e8071c91299852a07a84c4eb85007476338b7547ce8"

echo "Storing SEFI"
resp=$(secretcli tx compute store "${wasm_path}/gov_token.wasm" --from "$deployer_name" --gas 3000000 -b block -y)
echo $resp
token_code_id=$(echo $resp | jq -r '.logs[0].events[0].attributes[] | select(.key == "code_id") | .value')
#token_code_hash=$(secretcli query compute list-code | jq -r '.[-1]."data_hash"')
echo "Stored token: '$token_code_id', '$token_code_hash'"

echo "Storing Weight Master"
resp=$(secretcli tx compute store "${wasm_path}/weight_master.wasm" --from "$deployer_name" --gas 3000000 -b block -y)
echo $resp
master_code_id=$(echo $resp | jq -r '.logs[0].events[0].attributes[] | select(.key == "code_id") | .value')
#master_code_hash=$(secretcli query compute list-code | jq -r '.[-1]."data_hash"')
echo "Stored master: '$master_code_id'"

echo "Storing LP Staking"
resp=$(secretcli tx compute store "${wasm_path}/lp_staking.wasm" --from "$deployer_name" --gas 3000000 -b block -y)
echo $resp
lp_staking_code_id=$(echo $resp | jq -r '.logs[0].events[0].attributes[] | select(.key == "code_id") | .value')
#lp_staking_hash=$(secretcli query compute list-code | jq -r '.[-1]."data_hash"')
echo "Stored lp staking: '$lp_staking_code_id', '$lp_staking_hash'"

echo "Deploying Gov Token.."
export TX_HASH=$(
  secretcli tx compute instantiate $token_code_id '{"admin": "'$deployer_address'", "symbol": "SEFI", "decimals": 6, "initial_balances": [], "prng_seed": "YWE=", "name": "SEFI", "config": { "public_total_supply": true }}' --from $deployer_name --gas 1500000 --label SEFI-$revision -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH
gov_addr=$(secretcli query compute list-contract-by-code $token_code_id | jq -r '.[-1].address')
echo "SEFI address: '$gov_addr'"

token_code_hash="$(secretcli q compute contract-hash "$gov_addr")"
token_code_hash="${token_code_hash:2}"

echo "Deploying Master Contract.."
export TX_HASH=$(
  secretcli tx compute instantiate $master_code_id '{"gov_token_addr":"'"$gov_addr"'","gov_token_hash":"'"$token_code_hash"'","minting_schedule":[{"end_block":10000000,"mint_per_block":"100000000"}]}' --from $deployer_name --gas 1500000 --label MASTER-$revision -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH
master_addr=$(secretcli query compute list-contract-by-code $master_code_id | jq -r '.[-1].address')
echo "Master address: '$master_addr'"

master_code_hash="$(secretcli q compute contract-hash "$master_addr")"
master_code_hash="${master_code_hash:2}"

#echo "Deploying LP Token.."
#export TX_HASH=$(
#  secretcli tx compute instantiate $token_code_id '{"admin": "'$deployer_address'", "symbol": "LPT", "decimals": 6, "initial_balances": [{"address": "'$deployer_address'", "amount": "100000000000000000000000"}], "prng_seed": "YWE=", "name": "LPT"}' --from $deployer_name --gas 1500000 --label LP -b block -y |
#  jq -r .txhash
#)
#wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
#secretcli q compute tx $TX_HASH
#lp_token_addr=$(secretcli query compute list-contract-by-code $token_code_id | jq -r '.[-1].address')
#echo "LP Token address: '$lp_token_addr'"

echo "Deploying LP Staking 1 Contract.."
export TX_HASH=$(
  secretcli tx compute instantiate $lp_staking_code_id '{"reward_token":{"address":"'"$gov_addr"'", "contract_hash":"'"$token_code_hash"'"},"inc_token":{"address":"'"$lp_token1"'", "contract_hash":"'"$lp_token_hash"'"},"master":{"address":"'"$master_addr"'", "contract_hash":"'"$master_code_hash"'"},"viewing_key":"'"$viewing_key"'","token_info":{"name":"lps","symbol":"LPSTAKING"},"prng_seed":"YWE="}' --from $deployer_name --gas 1500000 --label sscrt-seth-lpstake-$revision -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH
lp_staking1_addr=$(secretcli query compute list-contract-by-code $lp_staking_code_id | jq -r '.[-1].address')

lp_staking_hash="$(secretcli q compute contract-hash "$lp_staking1_addr")"
lp_staking_hash="${lp_staking_hash:2}"

echo "Deploying LP Staking 2 Contract.."
export TX_HASH=$(
  secretcli tx compute instantiate $lp_staking_code_id '{"reward_token":{"address":"'"$gov_addr"'", "contract_hash":"'"$token_code_hash"'"},"inc_token":{"address":"'"$lp_token2"'", "contract_hash":"'"$lp_token_hash"'"},"master":{"address":"'"$master_addr"'", "contract_hash":"'"$master_code_hash"'"},"viewing_key":"'"$viewing_key"'","token_info":{"name":"lps","symbol":"LPSTAKING"},"prng_seed":"YWE="}' --from $deployer_name --gas 1500000 --label sscrt-scrt-lpstake-$revision -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH
lp_staking2_addr=$(secretcli query compute list-contract-by-code $lp_staking_code_id | jq -r '.[-1].address')

echo "Add Master as a SEFI minter"
export TX_HASH=$(
  secretcli tx compute execute "$gov_addr" '{"add_minters":{"minters":["'"$master_addr"'"]}}' --from $deployer_name --gas 1500000 -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH

echo "Setting LP Staking weight.."
export TX_HASH=$(
  secretcli tx compute execute "$master_addr" '{"set_weights":{"weights":[{"address":"'"$lp_staking1_addr"'","hash":"'"$lp_staking_hash"'","weight":33},{"address":"'"$lp_staking2_addr"'","hash":"'"$lp_staking_hash"'","weight":66}]}}' --from $deployer_name --gas 1500000 -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH

echo "Master address: '$master_addr'"
echo "SEFI address: '$gov_addr'"
echo "LP Staking 1 address: '$lp_staking1_addr'"
echo "LP Staking 2 address: '$lp_staking2_addr'"

#echo "Sleeping 10 sec.."
#sleep 10
#
#echo "Staking LP Token.."
#deposit_msg=$(base64 <<< '{"deposit":{}}')
#export TX_HASH=$(
#  secretcli tx compute execute "$lp_token_addr" '{"send":{"recipient":"'"$lp_staking_addr"'","amount":"1000","msg":"'"$deposit_msg"'"}}' --from $deployer_name --gas 1500000 -b block -y |
#  jq -r .txhash
#)
#wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
#secretcli q compute tx $TX_HASH
#
#echo "Setting viewing keys.."
#secretcli tx snip20 set-viewing-key $gov_addr "123" -b block --from $deployer_name -y
#secretcli tx snip20 set-viewing-key $lp_token_addr "123" -b block --from $deployer_name -y
#secretcli tx snip20 set-viewing-key $lp_staking_addr "123" -b block --from $deployer_name -y

#echo "a address: $deployer_address"

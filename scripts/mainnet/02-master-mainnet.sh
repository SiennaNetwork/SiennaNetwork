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
export prng_seed="gXBwdMGcNVbC8CtqptkwZeBwRScR7m5+LNbTrxoTHx4hLajYhrjIkmpaFLFXKStJTUT6mMS6wLGcUAkYSCPyfQ=="
export gov_addr="secret15l9cqgz5uezgydrglaak5ahfac69kmx2qpd6xt"
export gov_code_hash="c7fe67b243dfedc625a28ada303434d6f5a46a3086e7d2b5063a814e9f9a379d"
echo "Deployer address: '$deployer_address'"

echo "Storing Weight Master"
resp=$(secretcli tx compute store "${wasm_path}/weight_master.wasm" --from "$deployer_name" --gas 2000000 -b block -y)
echo $resp
master_code_id=$(echo $resp | jq -r '.logs[0].events[0].attributes[] | select(.key == "code_id") | .value')
echo "Stored master: '$master_code_id'"

echo "Deploying Master Contract.."
export TX_HASH=$(
  secretcli tx compute instantiate $master_code_id '{"gov_token_addr":"'"$gov_addr"'","gov_token_hash":"'"$gov_code_hash"'","minting_schedule":[{"end_block":7916452,"mint_per_block":"94368341"},{"end_block":13002903,"mint_per_block":"47184170"},{"end_block":18089355,"mint_per_block":"23592085"},{"end_block":23175806,"mint_per_block":"11796043"}]}' --from $deployer_name --gas 1500000 --label spy-master -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH
master_addr=$(secretcli query compute list-contract-by-code $master_code_id | jq -r '.[-1].address')
echo "Master address: '$master_addr'"

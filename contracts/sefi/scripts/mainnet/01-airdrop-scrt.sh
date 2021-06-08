#!/bin/bash

set -e

function wait_for_tx() {
  until (secretcli q tx "$1"); do
      sleep 5
  done
}

export wasm_path=/home/toml/Dev/GovAirdrop/secret-contracts/merkle-distributor

export deployer_name=sefi
export deployer_address=$(secretcli keys show -a $deployer_name)
export prng_seed="gXBwdMGcNVbC8CtqptkwZeBwRScR7m5+LNbTrxoTHx4hLajYhrjIkmpaFLFXKStJTUT6mMS6wLGcUAkYSCPyfQ=="
export sefi_addr="secret15l9cqgz5uezgydrglaak5ahfac69kmx2qpd6xt"
export sefi_hash="c7fe67b243dfedc625a28ada303434d6f5a46a3086e7d2b5063a814e9f9a379d"
export merkle_root="804c09bdcf4e06ffb53faafd12e6907e390dec3cc9fed2d5524c5f9a07d3c040"
export start_time=1617192000
echo "Deployer address: '$deployer_address'"

echo "Storing airdrop contract"
secretcli tx compute store "${wasm_path}/contract.wasm.gz" --from "$deployer_name" --gas 2000000 -b block -y
airdrop_code_id=$(secretcli query compute list-code | jq -r '.[-1]."id"')
echo "Stored airdrop contract: '$airdrop_code_id'"

echo "Deploying airdrop contract.."
export TX_HASH=$(
  secretcli tx compute instantiate $airdrop_code_id '{"token_addr":"'"$sefi_addr"'","token_hash":"'"$sefi_hash"'", "merkle_root":"'"$merkle_root"'", "start_time":'"$start_time"'}' --from $deployer_name --gas 1500000 --label sairdrop -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH
sefi_addr=$(secretcli query compute list-contract-by-code $token_code_id | jq -r '.[-1].address')
echo "airdrop contract address: '$sefi_addr'"
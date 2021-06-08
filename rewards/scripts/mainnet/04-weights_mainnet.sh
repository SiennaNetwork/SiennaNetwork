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
export lp_staking_hash="c0685e563cf038e6bc1e00c4187c8f502a96a5e393fd3c4b18a1367f0b083d7d"
export master_addr="secret1lyvgfgp48rj7skawfdjtehs23hfzdtj7kyrfwj"
echo "Deployer address: '$deployer_address'"

# spys and weights
#export spy_scrt_sefi="secret1097s3zmexc4mk9s2rdv3gs6r76x9dn9rmv86c7"
#export spy_scrt_sefi_weight=1000
#export spy_scrt_eth="secret146dg4c7jt5y37nw94swp6sahleshefxhrerpqm"
#export spy_scrt_eth_weight=400
#export spy_scrt_wbtc="secret1a3qvtsxd3fu5spkrscp5wwz3gtjmf50fgruezy"
#export spy_scrt_wbtc_weight=400
#export spy_scrt_usdt="secret1zysw570u5edsfdp44q80tm5zhdllawgh603ezy"
#export spy_scrt_usdt_weight=400
#export spy_eth_wbtc="secret1mznq6qwlj3ryzfpetfgydffef7w40tmlkhufcl"
#export spy_eth_wbtc_weight=400
#export spy_sefi="secret1y9z3ck449a46r4ku7klkhdxnlq07zh4shc7cuy"
#export spy_sefi_weight=709
export spy_scrt_link="secret1mlv3av6nlqt3fmzmtw0pnehsff2dxrzxq98225"
export spy_scrt_link_weight=100

#echo "Setting LP Staking weight.."
#export TX_HASH=$(
#  secretcli tx compute execute "$master_addr" '{"set_weights":{"weights":[{"address":"'"$spy_scrt_sefi"'","hash":"'"$lp_staking_hash"'","weight":'"$spy_scrt_sefi_weight"'},{"address":"'"$spy_scrt_eth"'","hash":"'"$lp_staking_hash"'","weight":'"$spy_scrt_eth_weight"'},{"address":"'"$spy_scrt_wbtc"'","hash":"'"$lp_staking_hash"'","weight":'"$spy_scrt_wbtc_weight"'},{"address":"'"$spy_scrt_usdt"'","hash":"'"$lp_staking_hash"'","weight":'"$spy_scrt_usdt_weight"'},{"address":"'"$spy_eth_wbtc"'","hash":"'"$lp_staking_hash"'","weight":'"$spy_eth_wbtc_weight"'},{"address":"'"$spy_sefi"'","hash":"'"$lp_staking_hash"'","weight":'"$spy_sefi_weight"'}]}}' --from $deployer_name --gas 1500000 -b block -y |
#  jq -r .txhash
#)
#wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
#secretcli q compute tx $TX_HASH

echo "Setting LP Staking weight.."
export TX_HASH=$(
  secretcli tx compute execute "$master_addr" '{"set_weights":{"weights":[{"address":"'"$spy_scrt_link"'","hash":"'"$lp_staking_hash"'","weight":'"$spy_scrt_link_weight"'}]}}' --from $deployer_name --gas 1500000 -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH

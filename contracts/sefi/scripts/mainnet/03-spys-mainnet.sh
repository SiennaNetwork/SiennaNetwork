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
export viewing_key="api_key_0ddf91b8f7a96314e01ff3ebe6fb20cf"
export prng_seed="gXBwdMGcNVbC8CtqptkwZeBwRScR7m5+LNbTrxoTHx4hLajYhrjIkmpaFLFXKStJTUT6mMS6wLGcUAkYSCPyfQ=="
echo "Deployer address: '$deployer_address'"

# master
export master_addr="secret1lyvgfgp48rj7skawfdjtehs23hfzdtj7kyrfwj"
export master_hash="c8555c2de49967ca484ba21cf563c2b27227a39ad6f32ff3de9758f20159d2d2"

# tokens
#export lp_scrt_sefi_token="secret1709qy2smh0r7jjac0qxfgjsqn7zpvgthsdz025"
#export lp_scrt_eth_token="secret17gja535zp09t9mlzzxndqqg4gzvhg0vsklhd54"
#export lp_scrt_wbtc_token="secret1xxvqanj85m7dppplku5782cn9hl8askqd329sv"
#export lp_scrt_usdt_token="secret1cgd6gcc4uyrxmzsmk4tpeta8auzcgwk4n5ngrx"
#export lp_eth_wbtc_token="secret1nvqrwwr9942gn89nk44nf2nku6gr7u8tsg6z45"
export lp_scrt_link_token="secret1rldr66767a4gz3adkq2vgndwgnxlfqqae4fgen"
export lp_token_hash="ea3df9d5e17246e4ef2f2e8071c91299852a07a84c4eb85007476338b7547ce8"

export sefi_token="secret15l9cqgz5uezgydrglaak5ahfac69kmx2qpd6xt"
export sefi_hash="c7fe67b243dfedc625a28ada303434d6f5a46a3086e7d2b5063a814e9f9a379d"

export lp_staking_code_id=38

#echo "Storing LP Staking"
#resp=$(secretcli tx compute store "${wasm_path}/lp_staking.wasm" --from "$deployer_name" --gas 3000000 -b block -y)
#echo $resp
#lp_staking_code_id=$(echo $resp | jq -r '.logs[0].events[0].attributes[] | select(.key == "code_id") | .value')
#echo "Stored lp staking: '$lp_staking_code_id'"

## spy-scrt-sefi
#echo "Deploying spy-scrt-sefi Contract.."
#export TX_HASH=$(
#  secretcli tx compute instantiate $lp_staking_code_id '{"reward_token":{"address":"'"$sefi_token"'", "contract_hash":"'"$sefi_hash"'"},"inc_token":{"address":"'"$lp_scrt_sefi_token"'", "contract_hash":"'"$lp_token_hash"'"},"master":{"address":"'"$master_addr"'", "contract_hash":"'"$master_hash"'"},"viewing_key":"'"$viewing_key"'","token_info":{"name":"spy-scrt-sefi","symbol":"SPYSCRTSEFI"},"prng_seed":"'"$prng_seed"'"}' --from $deployer_name --gas 1500000 --label spy-scrt-sefi -b block -y |
#  jq -r .txhash
#)
#wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
#secretcli q compute tx $TX_HASH
#spy_scrt_sefi=$(secretcli query compute list-contract-by-code $lp_staking_code_id | jq -r '.[-1].address')
#
## spy-scrt-eth
#echo "Deploying spy-scrt-eth Contract.."
#export TX_HASH=$(
#  secretcli tx compute instantiate $lp_staking_code_id '{"reward_token":{"address":"'"$sefi_token"'", "contract_hash":"'"$sefi_hash"'"},"inc_token":{"address":"'"$lp_scrt_eth_token"'", "contract_hash":"'"$lp_token_hash"'"},"master":{"address":"'"$master_addr"'", "contract_hash":"'"$master_hash"'"},"viewing_key":"'"$viewing_key"'","token_info":{"name":"spy-scrt-eth","symbol":"SPYSCRTETH"},"prng_seed":"'"$prng_seed"'"}' --from $deployer_name --gas 1500000 --label spy-scrt-eth -b block -y |
#  jq -r .txhash
#)
#wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
#secretcli q compute tx $TX_HASH
#spy_scrt_eth=$(secretcli query compute list-contract-by-code $lp_staking_code_id | jq -r '.[-1].address')
#
## spy-scrt-wbtc
#echo "Deploying spy-scrt-wbtc Contract.."
#export TX_HASH=$(
#  secretcli tx compute instantiate $lp_staking_code_id '{"reward_token":{"address":"'"$sefi_token"'", "contract_hash":"'"$sefi_hash"'"},"inc_token":{"address":"'"$lp_scrt_wbtc_token"'", "contract_hash":"'"$lp_token_hash"'"},"master":{"address":"'"$master_addr"'", "contract_hash":"'"$master_hash"'"},"viewing_key":"'"$viewing_key"'","token_info":{"name":"spy-scrt-wbtc","symbol":"SPYSCRTWBTC"},"prng_seed":"'"$prng_seed"'"}' --from $deployer_name --gas 1500000 --label spy-scrt-wbtc -b block -y |
#  jq -r .txhash
#)
#wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
#secretcli q compute tx $TX_HASH
#spy_scrt_wbtc=$(secretcli query compute list-contract-by-code $lp_staking_code_id | jq -r '.[-1].address')
#
## spy-scrt-usdt
#echo "Deploying spy-scrt-usdt Contract.."
#export TX_HASH=$(
#  secretcli tx compute instantiate $lp_staking_code_id '{"reward_token":{"address":"'"$sefi_token"'", "contract_hash":"'"$sefi_hash"'"},"inc_token":{"address":"'"$lp_scrt_usdt_token"'", "contract_hash":"'"$lp_token_hash"'"},"master":{"address":"'"$master_addr"'", "contract_hash":"'"$master_hash"'"},"viewing_key":"'"$viewing_key"'","token_info":{"name":"spy-scrt-usdt","symbol":"SPYSCRTUSDT"},"prng_seed":"'"$prng_seed"'"}' --from $deployer_name --gas 1500000 --label spy-scrt-usdt -b block -y |
#  jq -r .txhash
#)
#wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
#secretcli q compute tx $TX_HASH
#spy_scrt_usdt=$(secretcli query compute list-contract-by-code $lp_staking_code_id | jq -r '.[-1].address')
#
## spy-eth-wbtc
#echo "Deploying spy-eth-wbtc Contract.."
#export TX_HASH=$(
#  secretcli tx compute instantiate $lp_staking_code_id '{"reward_token":{"address":"'"$sefi_token"'", "contract_hash":"'"$sefi_hash"'"},"inc_token":{"address":"'"$lp_eth_wbtc_token"'", "contract_hash":"'"$lp_token_hash"'"},"master":{"address":"'"$master_addr"'", "contract_hash":"'"$master_hash"'"},"viewing_key":"'"$viewing_key"'","token_info":{"name":"spy-eth-wbtc","symbol":"SPYETHWBTC"},"prng_seed":"'"$prng_seed"'"}' --from $deployer_name --gas 1500000 --label spy-eth-wbtc -b block -y |
#  jq -r .txhash
#)
#wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
#secretcli q compute tx $TX_HASH
#spy_eth_wbtc=$(secretcli query compute list-contract-by-code $lp_staking_code_id | jq -r '.[-1].address')
#
## spy-sefi
#echo "Deploying spy-sefi Contract.."
#export TX_HASH=$(
#  secretcli tx compute instantiate $lp_staking_code_id '{"reward_token":{"address":"'"$sefi_token"'", "contract_hash":"'"$sefi_hash"'"},"inc_token":{"address":"'"$sefi_token"'", "contract_hash":"'"$sefi_hash"'"},"master":{"address":"'"$master_addr"'", "contract_hash":"'"$master_hash"'"},"viewing_key":"'"$viewing_key"'","token_info":{"name":"spy-sefi","symbol":"SPYSEFI"},"prng_seed":"'"$prng_seed"'"}' --from $deployer_name --gas 1500000 --label spy-sefi -b block -y |
#  jq -r .txhash
#)
#wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
#secretcli q compute tx $TX_HASH
#spy_sefi=$(secretcli query compute list-contract-by-code $lp_staking_code_id | jq -r '.[-1].address')

# spy-scrt-link
echo "Deploying spy-scrt-link Contract.."
export TX_HASH=$(
  secretcli tx compute instantiate $lp_staking_code_id '{"reward_token":{"address":"'"$sefi_token"'", "contract_hash":"'"$sefi_hash"'"},"inc_token":{"address":"'"$lp_scrt_link_token"'", "contract_hash":"'"$lp_token_hash"'"},"master":{"address":"'"$master_addr"'", "contract_hash":"'"$master_hash"'"},"viewing_key":"'"$viewing_key"'","token_info":{"name":"spy-scrt-link","symbol":"SPYSCRTLINK"},"prng_seed":"'"$prng_seed"'"}' --from $deployer_name --gas 1500000 --label spy-scrt-link -b block -y |
  jq -r .txhash
)
wait_for_tx "$TX_HASH" "Waiting for tx to finish on-chain..."
secretcli q compute tx $TX_HASH
spy_scrt_link=$(secretcli query compute list-contract-by-code $lp_staking_code_id | jq -r '.[-1].address')

echo "Addresses:"
#echo "spy-scrt-sefi: '$spy_scrt_sefi'"
#echo "spy-scrt-eth: '$spy_scrt_eth'"
#echo "spy-scrt-wbtc: '$spy_scrt_wbtc'"
#echo "spy-scrt-usdt: '$spy_scrt_usdt'"
#echo "spy-eth-wbtc: '$spy_eth_wbtc'"
#echo "spy-sefi: '$spy_sefi'"
echo "spy-scrt-link: '$spy_scrt_link'"



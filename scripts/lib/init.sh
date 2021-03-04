#!/bin/bash
cd ~
whoami
pwd
ls -alh
file=~/.secretd/config/genesis.json
if [ ! -e "$file" ]; then
  echo "clear state from preceding genesis ----"
  rm -rf ~/.secretd/*
  rm -rf ~/.secretcli/*
  rm -rf ~/.sgx_secrets/*
  echo "initialize secretcli-------------------"
  secretcli config chain-id enigma-pub-testnet-3
  secretcli config output json
  secretcli config indent true
  secretcli config trust-node true
  secretcli config keyring-backend test
  echo "initialize secretd---------------------"
  secretd init banana --chain-id enigma-pub-testnet-3
  cp ~/node_key.json ~/.secretd/config/node_key.json
  perl -i -pe 's/"stake"/"uscrt"/g' ~/.secretd/config/genesis.json # wtf is going on here
  echo "create admin key-----------------------"
  ADMIN="ADMIN"
  ADMIN_KEY=`secretcli keys add $ADMIN 2>&1`
  echo "$ADMIN_KEY" > /shared-keys/admin_key.json
  cat /shared-keys/admin_key.json
  chmod a+r /shared-keys/admin_key.json
  echo "get admin address----------------------"
  ADMIN_ADDR="$(secretcli keys show -a $ADMIN)"
  echo "$ADMIN_ADDR"
  echo "add genesis balance for admin----------"
  secretd add-genesis-account "$ADMIN_ADDR" 1000000000000000000uscrt
  secretd gentx --name $ADMIN --keyring-backend test --amount 1000000uscrt
  echo "mystery block 1------------------------"
  secretd collect-gentxs
  secretd validate-genesis
  echo "mystery block 2------------------------"
  secretd init-bootstrap
  secretd validate-genesis
fi

secretcli rest-server --trust-node=true --chain-id enigma-pub-testnet-3 --laddr tcp://0.0.0.0:1336 &
lcp --proxyUrl http://localhost:1336 --port 1337 --proxyPartial '' &

# sleep infinity
source /opt/sgxsdk/environment && \
  RUST_BACKTRACE=1 secretd start --rpc.laddr tcp://0.0.0.0:26657 --bootstrap

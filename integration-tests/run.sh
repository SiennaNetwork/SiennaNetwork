docker run -d --rm \
 -p 26657:26657 -p 26656:26656 -p 1337:1337 \
 --name secretdev enigmampc/secret-network-sw-dev

Commit=`git rev-parse --short HEAD`

sleep 1

Keys=$(docker exec secretdev /bin/bash -c "secretcli keys list --keyring-backend test")

node --trace-warnings index.js $Commit "$Keys"

docker kill secretdev
 
if ! npx tsc -p . ; then
    exit
fi

docker create --rm \
 -p 26657:26657 -p 26656:26656 -p 1337:1337 \
 --name secretdev enigmampc/secret-network-sw-dev:v1.2.2-1

docker cp $(pwd)/bootstrap_init.sh secretdev:/root/bootstrap_init.sh

docker start secretdev

# If you get an error like "Account does not exist on chain. Send some tokens there before trying to query nonces."
# try to increase this time
sleep 8

Keys=$(docker exec secretdev /bin/bash -c "cat /root/the_keys.txt")

node_modules/.bin/esmo --enable-source-maps ./simulation.ts "$Keys"

docker kill secretdev

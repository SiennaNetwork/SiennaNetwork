if ! npx tsc -p . ; then
    exit
fi

sudo docker create --rm \
 -p 26657:26657 -p 26656:26656 -p 1337:1337 \
 --name secretdev enigmampc/secret-network-sw-dev

sudo docker cp $(pwd)/bootstrap_init.sh secretdev:/root/bootstrap_init.sh

sudo docker start secretdev

# If you get an error like "Account does not exist on chain. Send some tokens there before trying to query nonces."
# try to increase this time
sleep 8

Commit=`git rev-parse --short HEAD`

Keys=$(sudo docker exec secretdev /bin/bash -c "cat /root/the_keys.json")

node --trace-warnings ./dist/index.js $Commit "$Keys"

sudo docker kill secretdev
 
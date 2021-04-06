if ! npx tsc -p . ; then
    exit
fi

sudo docker create --rm \
 -p 26657:26657 -p 26656:26656 -p 1337:1337 \
 --name secretdev enigmampc/secret-network-sw-dev

sudo docker cp $(pwd)/bootstrap_init.sh secretdev:/root/bootstrap_init.sh

sudo docker start secretdev

sleep 5

Commit=`git rev-parse --short HEAD`

Keys=$(sudo docker exec secretdev /bin/bash -c "cat /root/the_keys.json")

node --trace-warnings ./dist/index.js $Commit "$Keys"

sudo docker kill secretdev
 
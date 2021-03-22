docker run -d --rm \
 -p 26657:26657 -p 26656:26656 -p 1337:1337 \
 --name secretdev enigmampc/secret-network-sw-dev

 Commit=`git rev-parse --short HEAD`

 node --trace-warnings index.js $Commit

 docker kill secretdev
 
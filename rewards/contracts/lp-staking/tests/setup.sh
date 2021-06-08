#!/bin/bash

# Start docker
make start-server-detached

# Transfer secret-secret inside
docker exec -it secretdev mkdir secret-secret
docker cp tests/secret-secret/contract.wasm.gz secretdev:/root/secret-secret/

sleep 20

make run-tests
if [ $? -eq 0 ]
then
  echo "Tests passed successfully!"
  exit_status=0
else
  echo "Tests failed!" >&2
  exit_status=1
fi

docker stop secretdev

exit $exit_status
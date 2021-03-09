* This directory contains a modification
  of the init script for `enigmampc/secret-network-sw-dev`
  that stores the genesis keys in a volume
  (provided by `docker-compose`, called `shared-keys`).
  * This is mounted by the `compare` container
    in order to have balance to run the integration tests.

* To start a localnet,
  go back to the root of the repository,
  and run `docker-compose up -d localnet`.
  * Then, to run integration tests,
    run `docker-cmpose run compare`.

* `unroot.sh` is an attempt to run the local node
  without root (however this seems impossible
  because of SGX?)

# Receipts

These are used by subclasses of `BaseChain` (such as `Scrt`)
to provide an unified interface to the history of deployment
transactions and the associated resources.

## Structure

Several types of things are kept around as JSON documents:

* `$CHAIN_ID/identities/`: private keys for service accounts
  (no need to recreate wallets)

* `$CHAIN_ID/uploades/`: outputs of code upload transactions
  (no need to reupload contracts)

* `$CHAIN_ID/instances/$DATE`: outputs of contract instantiations
  (keep track of instantiated contracts)

* `$CHAIN_ID/instances/.active`: symlink to the current instance directory
  (select a group of contracts, such as "the production deployment" or
  "the latest testnet deployment" to use as a target for further operations)

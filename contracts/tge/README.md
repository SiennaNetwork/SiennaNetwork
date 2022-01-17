# Sienna TGE/Vesting

## Contents

* `contracts/snip20-sienna`
* `contracts/mgmt`
* `contracts/rpt`

## Run tests

```sh
cargo test -p snip20-sienna
cargo test -p mgmt
cargo test -p rpt
```

## Compile for production

```sh
pnpm -w dev build tge
```

## Configure

* MGMT can be reconfigured by its admin after deployment
  as long as it hasn't been launched yet.

* RPT can be freely reconfigured by its admin
  as long as the budget adds up to the original amount (2500 SIENNA).

## Use

* To claim funds from MGMT, send it `{"claim":{}}`.
* To make RPT send funds to the reward pools, send it `{"vest":{}}`

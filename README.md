# SIENNA

* `docs`      - documentation generation
* `deployer`  - deploy scripts
* `optimizer` - build tool
* `token`     - standard SNIP20 token
* `mgmt`      - vesting management contract
* `fadroma`   - smart contract macro library
* `kukumba`   - BDD macro library

## Quick start

Make sure you're familiar with the `docs`, then use the `deployer` to
upload a build (produced by the `optimizer`) of the `token` and `mgmt`
contracts (the latter of which is built with `fadroma` and tested with 
`kukumba`).

### Obtain, build, and verify the code

```sh
git clone --recurse-submodules git@github.com:hackbg/sienna-secret-token.git sienna \
  && cd sienna \
  && make test \
  && make \
  && ls dist/ \
  && cat dist/checksums.sha256.txt
```

### Prepare for deployment

Create a file called `.env` in the repository root, and populate it with
the node URL, the chain ID, and your mnemonic:

```sh
cp env.example .env
$EDITOR .env
```

### Deploy

To run the deployer in a Docker containter:
```sh
make deploy
```

To run the deployer outside of a Docker container:
```sh
./deployer/deploy.js
```

## Vesting schedule

![](docs/schedule.svg)

# SIENNA

* `docs`      - documentation generation
* `deployer`  - deploy scripts
* `optimizer` - build tool
* `token`     - standard SNIP20 token
* `mgmt`      - vesting management contract
* `schedule`  - library implementing vesting schedule logic
* `fadroma`   - smart contract macro library
* `kukumba`   - BDD macro library

## Quick start

Make sure you're familiar with the `docs`, then use the `deployer` to
upload a build (produced by the `optimizer`) of the `token` and `mgmt`
contracts (the latter of which is built with `fadroma`, tested with 
`kukumba`, and configured with a `schedule`).

### Fetch, build, and verify the code

```sh
git clone --recurse-submodules git@github.com:hackbg/sienna-secret-token.git sienna \
  && cd sienna \     # enter repository
  && make test \     # run tests
  && make coverage \ # generate test coverage reports
  && make \          # production build
  && ls dist/ \      # view contents of production build
  && cat dist/checksums.sha256.txt # view production build checksums
```

### Configure the contract

The file `config.json` is used to configure the contract before launching.
To generate it:

* Go to [the spreadsheet](https://docs.google.com/spreadsheets/d/1sgj-nTE_b25F8O740Av7XYByOzkD0qNx1Jk63G2qRwY/)
  that defines the vesting schedule.
* Download it as TSV using **File->Download->Tab-separated values (.tsv, current sheet)**
* Replace `schedule.tsv` with the downloaded file (renaming it from e.g. `SIENNA - Schedule.tsv`)
* Run `make config` to obtain an up-to-date `config.json`
* FIXME: Run `make chart` to visualize the vesting schedule.

### Prepare for deployment

Create a file called `.env` in the repository root, and populate it with
the node URL, the chain ID, and your mnemonic:

```sh
cp env.example .env
$EDITOR .env
```

### Deploy

First, make sure you have a valid `config.json`.

To run the deployer in a Docker containter:
```sh
make deploy
```

To run the deployer outside of a Docker container:
```sh
./deployer/deploy.js
```

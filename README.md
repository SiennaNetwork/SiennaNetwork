# SIENNA

## Quick start

Here's how to fetch the code, install JS dependencies,
and obtain a list of the actions you can perform:

```sh
git clone --recurse-submodules git@github.com:hackbg/sienna-secret-token.git sienna 
cd sienna
yarn
./sienna.js --help
```

```
sienna.js <command>

Commands:
  sienna.js docs [crate]     📖 Build the documentation and open it in a browser.
  sienna.js test             ⚗️  Run test suites for all the individual components.
  sienna.js coverage         🗺️  Generate test coverage and open it in a browser.
  sienna.js demo             📜 Run integration tests/demos/executable reports.
  sienna.js schema           🤙 Regenerate JSON schema for each contract's API.
  sienna.js schedule [file]  📅 Convert a spreadsheet into a JSON schedule for the contract.
  sienna.js build [ref]      👷 Compile all contracts - either from working tree or a Git ref
  sienna.js deploy           🚀 Upload, instantiate, and configure all contracts.
  sienna.js launch           💸 Launch the vesting contract.

Options:
  --help     Show help                                                              [boolean]
  --version  Show version number                                                    [boolean]
```

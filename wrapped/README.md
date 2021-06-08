# Wrapped Sienna (ERC20) Token

# Run locally

Clone the repo and:

## Install dependencies

```
$ npm install
```

## Compile

```
$ npx truffle compile
```

## Test

```
$ npx truffle test
```

Output (example):
```

  Contract: WrappedSienna
    ✓ has a name
    ✓ has a symbol
    ✓ has decimals
    ✓ paused is successfull (30447 gas)
    ✓ unpaused is successfull (60891 gas)
    ✓ not allowed to pause from account with no pauser role (22581 gas)
    ✓ not allowed to unpause from account with no pauser role (53028 gas)
    ✓ transfer tokens from minter to other account (mint) (68245 gas)
    ✓ transfer tokens from other account to minter (burn) (107718 gas)
    ✓ transfer tokens from one account to another (122944 gas)
    ✓ not allowed to transfer tokens during the contract is paused (124274 gas)

·----------------------------------------|----------------------------|-------------|----------------------------·
|  Solc version: 0.6.12+commit.27d51765  ·  Optimizer enabled: false  ·  Runs: 200  ·  Block limit: 6718946 gas  │
·········································|····························|·············|·····························
|  Methods                                                                                                       │
·······················|·················|··············|·············|·············|··············|··············
|  Contract            ·  Method         ·  Min         ·  Max        ·  Avg        ·  # calls     ·  eur (avg)  │
·······················|·················|··············|·············|·············|··············|··············
|  WrappedSienna       ·  pause          ·           -  ·          -  ·      30447  ·           5  ·          -  │
·······················|·················|··············|·············|·············|··············|··············
|  WrappedSienna       ·  transfer       ·       39473  ·      68245  ·      58841  ·           9  ·          -  │
·······················|·················|··············|·············|·············|··············|··············
|  WrappedSienna       ·  unpause        ·           -  ·          -  ·      30444  ·           2  ·          -  │
·······················|·················|··············|·············|·············|··············|··············
|  Deployments                           ·                                          ·  % of limit  ·             │
·········································|··············|·············|·············|··············|··············
|  WrappedSienna                         ·           -  ·          -  ·    2417607  ·        36 %  ·          -  │
·----------------------------------------|--------------|-------------|-------------|--------------|-------------·

  11 passing (8s)
```

## Coverage report

```
$ npx truffle run coverage
```

Output (example):
```
  Contract: WrappedSienna
    ✓ has a name
    ✓ has a symbol
    ✓ has decimals
    ✓ paused is successfull (57ms)
    ✓ unpaused is successfull (144ms)
    ✓ not allowed to pause from account with no pauser role (745ms)
    ✓ not allowed to unpause from account with no pauser role (135ms)
    ✓ transfer tokens from minter to other account (mint) (212ms)
    ✓ transfer tokens from other account to minter (burn) (306ms)
    ✓ transfer tokens from one account to another (169ms)
    ✓ not allowed to transfer tokens during the contract is paused (152ms)


  11 passing (4s)

--------------------|----------|----------|----------|----------|----------------|
File                |  % Stmts | % Branch |  % Funcs |  % Lines |Uncovered Lines |
--------------------|----------|----------|----------|----------|----------------|
 contracts/         |      100 |      100 |      100 |      100 |                |
  WrappedSienna.sol |      100 |      100 |      100 |      100 |                |
--------------------|----------|----------|----------|----------|----------------|
All files           |      100 |      100 |      100 |      100 |                |
--------------------|----------|----------|----------|----------|----------------|

> Istanbul reports written to ./coverage/ and ./coverage.json
> solidity-coverage cleaning up, shutting down ganache server
```

## Deployment

1. Create a `.secret` file which contains the mnemonic phrase for the account which is to be used for deployment.
2. Make sure you've set the proper bridge address in the deployment script here: https://github.com/SiennaNetwork/wrapped-sienna/blob/5c9e934c421c10d811aec7d2122683af15bac61d/migrations/2_deploy_contracts.js#L6
3. Make sure the Infura API key is working here: https://github.com/SiennaNetwork/wrapped-sienna/blob/5c9e934c421c10d811aec7d2122683af15bac61d/truffle-config.js#L22

```
$ npx truffle migrate --network <network>
```

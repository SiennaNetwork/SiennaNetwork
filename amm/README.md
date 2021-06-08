# Sienna Automated market-maker (AMM)

This is an Automated Marked Maker Decentralized Exchange written in Rust (CosmWasm) inspired by Uniswap Protocol.

## Contracts

### Exchange factory
Used to create exchange contracts between different tokens.

#### Responsibilities and constraints:
 * Creates new exchange contracts.
 * Can create only one exchange per token pair.
 * Stores all existing exchanges created.

### Exchange pair
Exchange contracts are automated market makers between a token pair. These can be either SCRT or a SNIP20 compliant token.

## Actors

### Trader
A trader can exchange their token for another token through SiennaSwap using the price determined by the liquidity pool ratio.

### Liquidity Provider
Must deposit an equivalent value of both tokens. This increases liquidity for the corresponding pair market while maintaining the pool price.


## References
- https://github.com/enigmampc/SecretSwap
- https://github.com/terraswap/terraswap
- https://github.com/Uniswap/uniswap-v1

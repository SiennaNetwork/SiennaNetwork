# Sienna Lend

This is the implementation for Sienna Lend, a privacy-preserving protocol for supplying or borrowing assets. The solution is inspired by [Compound Protocol]((https://compound.finance/docs)) and follows similar principles. Through Market contracts, accounts can supply capital (Snip20 tokens) to receive slTokens or borrow assets from the protocol (while holding a collateral). The market contract tracks the balances and algorithmically sets interest rates.

## **Contracts**
### Market
The function of this contract is to allow accounts to deposit (supply capital), redeem (withdraw capital), borrow and repay a loan. Liquidators also liquidate underwater loans through here. In addition, the market itself represents a token (we call it slToken) which is minted to depositors upon supplying the underling asset that the given market is for. There is one instance per asset. It partially implements the SNIP-20 interface, enough to support basic balance queries and transfers while also being compatible with the Keplr wallet. The exchange rate between the slToken and the underlying asset is not 1:1 but rather based on supply and demand. Functional equivalent to the [cToken](https://compound.finance/docs/ctokens) in Compound.
#### Repaying a loan
It's actually a common scenario when wanting to repay your loan in full. During the time between submitting the repay TX and it actually being executed, interest might be accrued thus causing the repay amount provided to not repay the loan in full. To prevent this users are forced to send a slightly larger repay amount to the market contract. Now any remainder is given back to the user when the `repay amount > borrow balance`. Note that this is not implemented for liquidations.

Example:
If a user has borrowed `100 SSCRT` and repays `101 SSCRT`, the contract will then send them back the change: `1 SSCRT`. This will prevent the loss of funds when repaying loans.

#### SL Tokens
- Implements the Send and RegisterReceive methods from the SNIP-20 spec. This is allowing depositing SL tokens into SiennaSwap rewards contracts.

### Overseer
The overseer contract validates permissible user actions. For instance, this contract enforces that each borrower must maintain a sufficient collateral balance across all slTokens. It is also responsible for whitelisting markets, determining whether and by how much a user can be liquidated etc. It also serves as a market factory as they are created through it by the admin. Functional equivalent to the [Comptroller](https://compound.finance/docs/comptroller) in Compound.

### InterestModel
Contract which defines interest rates. Its models determine interest rates based on the current utilization of a given market (how much of the supplied capital is liquid vs borrowed). It follows Compound Protocol's JumpRateModel.

### Oracle
This is the contract that is used for consuming prices from [Band Protocol's](https://bandprotocol.com/) price feeds.

---
Diagrams can be found in [docs/Sienna Lend.drawio.png](../../docs/Sienna%20Lend.drawio.png)

## **Scope**

All the relevant code is in the following locations:
 - Contracts: `contracts/lend`, excluding the `tests` and `mock_band_oracle` directories
 - Shared library:  `libraries/lend-shared`

## **Challenges**

 - Allowing private loans that don't expose user information.
 - Ensuring privacy while allowing the protocol internal access to everyting.

## **Notes**
 - The contracts are written using a [derive-contract](https://github.com/hackbg/fadroma/tree/22.01/crates/fadroma-derive-contract) macro from fadroma, message names (in PascalCase) are derived from method names (snake_case).
 - In order for the contracts to be able to read the necessary private data internally we have used what is called a [`MasterKey`](libraries/lend-shared/src/core/state.rs#L13). It is a single **viewing key** shared throughout the protocol. The MasterKey is only known by the contract internally and cannot be queried or extracted.
 - Some information (such as loans up for liquidation) has to be shared in order for the protocol to work as intended. In order for this information to be visible without knowing whose address it is tied to we have introduced `IDs` throughout the protocol. These IDs are issued by the Market contract once for each user (at the first borrow. This way liquidators can view relevant information for a loan without knowing who the loan is tied to. An ID is a [sha256 hash](contracts/lend/market/src/state.rs#L362) derived from an address + seed. Each market has its own private seed, issued during init, so a single user address will have a different ID in each market.
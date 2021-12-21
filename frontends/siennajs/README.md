# SiennaJS
Client library to interact with smart contracts on Sienna Network

## Usage
All smart contract interfaces are created with the following parameters.

 - The address of the contract.
 - An instance of `SigningCosmWasmClient` from `secretjs` (Optional).
 - An instance of `CosmWasmClient` from `secretjs` (Optional).

 At least one type of client is required. Depending on the type of client provided you will get access to different functionalities:

  - `SigningCosmWasmClient` - both executing transactions and queries
  - `CosmWasmClient` - queries only

If both instances are passed it will use `SigningCosmWasmClient` for executing and `CosmWasmClient` for queries.

## Example - Query SIENNA token info:

```typescript
const query_client = new CosmWasmClient('API_URL_HERE')
const sienna_token = new Snip20Contract(
    'secret1rgm2m5t530tdzyd99775n6vzumxa5luxcllml4',
    undefined, // We don't pass a SigningCosmWasmClient as we don't need it for queries
    query_client
)

const token_info = await sienna_token.query().get_token_info()
```

## Querying with permits
SiennaJS exposes the `Signer` interface which must implement an offline signing method. See the [SNIP-24 spec](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-24.md#data-structures) for more info.

An implementation for the [Keplr](https://www.keplr.app) wallet is provided by the library - `KeplrSigner`

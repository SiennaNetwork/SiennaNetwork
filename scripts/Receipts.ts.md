# Sienna Scripts: Import

Import receipts from transactions done outside these scripts.

```typescript
import Fadroma, { bold, timestamp, Console, Deployments } from '@hackbg/fadroma'
import * as API from '@sienna/api'
const console = new Console('@sienna/scripts/Import')
```

### Receipts

Results of uploads and inits are stored in `receipts/*/{deployments,uploads}`.
These are used to keep track of deployed contracts.
See [`../receipts`](../receipts).

#### Import exchange receipts from factory

Contract receipts are normally saved only when the contracts
are created via Fadroma's `Deployment` class. In comparison,
AMM exchanges are created by executing a transaction on the
AMM Factory contract, which does not emit receipts. Therefore,
in to keep track of exchanges, the following command is used
to query the current list of exchanges from the factory, and
generate the corresponding exchange contract receipts.

```typescript
Fadroma.command('import receipts amm v2',
  Deployments.activate,
  async function ammFactoryImportReceipts_v2 ({
    agent, deployment,
    factory = new AMMFactoryClient.v2({
      ...deployment.get(['AMM[v2].Factory']),
      agent
    })
  }) {
    const {
      pair_contract: { id: ammId, code_hash: ammHash },
      lp_token_contract: { id: lpId }
    } = await factory.getTemplates()
    const exchanges = await factory.listExchangesFull()
    for (const {name, raw} of exchanges) {
      deployment
        .add(`AMM[v2].${name}`, {
          ...raw,
          codeId:   ammId,
          codeHash: ammHash,
          address:  raw.exchange.address,
        })
        .add(`AMM[v2].${name}.LP`, {
          ...raw,
          codeId:   lpId,
          codeHash: raw.lp_token.code_hash,
          address:  raw.lp_token.address
        })
    }
  }
)
```

#### Saving exchange receipts from the factory

When deploying contracts from Fadroma Ops, receipts are created.
When deploying contracts from the factory contract, this does not happen.
These functions gets the created exchanges from the factory and
create the corresponding receipts.

```typescript
export async function saveCreatedPairs ({ deployment, FACTORY, ammVersion, createdPairs }) {
  const inventory = await FACTORY.getTemplates()
  const EXCHANGES = await Promise.all(createdPairs.map(async ([token0, token1])=>{
    const exchange = await FACTORY.getExchange(token0, token1)
    return saveAMMExchange({ deployment, ammVersion, inventory, exchange })
  }))
  return { EXCHANGES }
}

export async function saveExchangeReceipts (
  deployment,
  ammVersion: API.AMMVersion,
  factory:    API.AMMFactoryClient,
  pairs:      any[]
) {
  // turn the list of pairs to create
  // into a list of created exchange instances
  console.info('Saving receipts for pairs:', ...pairs)
  const exchanges = await pairsToExchanges(factory, pairs)
  const inventory = await factory.getTemplates()
  // save the newly created contracts to the deployment
  await Promise.all(exchanges.map((exchange)=>saveAMMExchange({
    deployment, ammVersion, inventory, exchange
  })))
}

function pairsToExchanges (factory, pairs) {
  return Promise.all(pairs.map(
    ({pair:{token_0, token_1, TOKEN_0, TOKEN_1}})=>factory.getExchange(
      token_0||TOKEN_0?.asCustomToken,
      token_1||TOKEN_1?.asCustomToken
    )
  ))
}

export async function saveAMMExchange ({
  deployment,
  ammVersion,
  inventory: {
    pair_contract: { id: ammId, code_hash: ammHash },
    lp_token_contract: { id: lpId }
  },
  exchange: { name, raw, EXCHANGE, LP_TOKEN, TOKEN_0, TOKEN_1 }
}) {
  //console.info(bold(`Deployed AMM exchange`), EXCHANGE.address)
  deployment.add(`AMM[${ammVersion}].${name}`, {
    ...raw,
    codeId:   ammId,
    codeHash: ammHash,
    address:  raw.exchange.address,
  })
  //console.info(bold(`Deployed LP token`), LP_TOKEN.address)
  deployment.add(`AMM[${ammVersion}].${name}.LP`, {
    ...raw,
    codeId:   lpId,
    codeHash: raw.lp_token.code_hash,
    address:  raw.lp_token.address
  })
  EXCHANGE.prefix = LP_TOKEN.prefix = deployment.prefix
  return { name, raw, EXCHANGE, LP_TOKEN, TOKEN_0, TOKEN_1 }
}
```

#### Import rewards receipts from bundle response

```typescript
Fadroma.command('import receipts rewards v3',
  Deployments.activate,
  async function importRewardsReceipts ({
    agent,
    deployment
  }) {
    const bundleReceiptPath = agent.chain.stateRoot.resolve('rewards-v3.json')
    const bundleReceiptData = JSON.parse(readFileSync(bundleReceiptPath, 'utf8'))
    const addresses = bundleReceiptData.logs.map(({ msg_index, log, events: [ message, wasm ] })=>{
      const address = message.attributes[4].value
      console.log(address)
      return address
    })
    const stakedTokens = new Map()
    const stakedTokenNames = new Map()
    const { codeId, codeHash } = agent.chain.uploads.load('sienna-rewards@39e87e4.wasm')
    await Promise.all(addresses.map(async address=>{
      const client = new RewardsClient.v3({ address, codeHash, agent })
      const label = await client.label
      deployment.add(label.split('/')[1], {
        label,
        codeId,
        codeHash,
        address,
        initTx: bundleReceiptData.txhash
      })
    }))
  }
  Deployments.status,
)
```

## Import receipts in old format

This function addes the minimum of
`{ codeId, codeHash, initTx: contractAddress }`
to AMM and Rewards pool instantiation receipts
from the mainnet deploy that were previously stored
in a non-compatible format.

```typescript
import * as Receipts from '@sienna/receipts'
Fadroma.command('fix 1', Receipts.fix1)
Fadroma.command('fix 2', Receipts.fix2)
```

## Entry point

```typescript
Error.stackTraceLimit = Infinity
export default Fadroma.module(import.meta.url)
```

# Sienna Scripts: Upload

```typescript
import Fadroma, { bold, timestamp, Console } from '@hackbg/fadroma'
const console = new Console('@sienna/scripts/Upload')
```

## Upload AMMv2 + Rewardsv3 contracts to mainnet

```typescript
import * as API from '@sienna/api'
Fadroma.command('amm-v2-rewards-v3', async ({ agent })=>{
  const [
    newAMMFactoryTemplate,
    newAMMExchangeTemplate,
    newAMMLPTokenTemplate,
    newRewardsTemplate
  ] = await agent.buildAndUpload([
    new API.AMMFactoryContract.v2()
    new API.AMMExchangeContract.v2()
    new API.LPTokenContract.v2()
    new API.RewardsContract.v3()
  ])
})
```

## Entry point

```typescript
Error.stackTraceLimit = Infinity
export default Fadroma.module(import.meta.url)
```

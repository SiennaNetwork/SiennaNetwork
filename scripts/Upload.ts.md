# Sienna Scripts: Upload

```typescript
import Fadroma, { bold, timestamp, Console } from '@hackbg/fadroma'
const console = new Console('@sienna/scripts/Upload')
```

> **Implementation:** [@fadroma/ops/Upload.ts](https://github.com/hackbg/fadroma/blob/v100/packages/ops/Upload.ts)

Uploads create `.json` files under `receipts/$FADROMA_CHAIN/uploads`.
If a upload receipt's code hash matches the one in the `.wasm.sha256`
for the corresponding contract, the upload becomes a no-op.

* **Env var:** `FADROMA_UPLOAD_ALWAYS` - always reupload the compiled contracts,
  even if an upload receipt is present

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

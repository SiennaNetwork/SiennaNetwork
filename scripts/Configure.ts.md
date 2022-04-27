# Sienna Scripts: Configuration

```typescript
import { Console, bold } from '@hackbg/fadroma'
const console = new Console('@sienna/scripts/Configure')
```

## Minting testnet tokens

* List of testers that are funded during deployment:

```typescript
```

## Adjusting the RPT config

After deploying the reward pools,
this function set their addresses in the RPT,
so that they receive funding from the daily vesting.

```typescript
import { RPTClient } from '@sienna/api'
export async function adjustRPTConfig ({
  deployment, chain, agent,
  RPT        = new RPTClient({ ...deployment.get('RPT'), agent }),
  RPT_CONFIG = [],
}) {
  // on mainnet we use a multisig
  // so we can't run the transaction from here
  if (chain.isMainnet) {
    deployment.save({config: RPT_CONFIG}, 'RPTConfig.json')
    console.info(
      `\n\nWrote RPT config to deployment ${deployment.prefix}. `+
      `You should use this file as the basis of a multisig transaction.`
    )
    return
  }
  console.info(bold(`Configuring RPT`), RPT.address)
  for (const [address, amount] of RPT_CONFIG) {
    console.info(` `, bold(amount), address)
  }
  await RPT.configure(RPT_CONFIG)
  return { RPT_CONFIG }
}
```

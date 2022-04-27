# Sienna Scripts: Configuration

```typescript
import { Console, bold } from '@hackbg/fadroma'
const console = new Console('@sienna/scripts/Configure')
```

## Minting testnet tokens

* List of testers that are funded during deployment:

```typescript
export const testers = [
  //"secret1vdf2hz5f2ygy0z7mesntmje8em5u7vxknyeygy",
  "secret13nkfwfp8y9n226l9sy0dfs0sls8dy8f0zquz0y",
  "secret1xcywp5smmmdxudc7xgnrezt6fnzzvmxqf7ldty",
]

import { SiennaSnip20Client } from '@sienna/api'
async function fundTesters ({ deployment, agent, cmdArgs }) {
  const [ vk = 'q1Y3S7Vq8tjdWXCL9dkh' ] = cmdArgs
  const sienna  = new SiennaSnip20Client({ ...deployment.get('SIENNA'), agent })
  const balanceBefore = await sienna.getBalance(agent.address, vk)
  console.info(`SIENNA balance of ${bold(agent.address)}: ${balanceBefore}`)
  const amount  = balanceBefore.slice(0, balanceBefore.length - 1)
  await sienna.transfer(amount, 'secret13nkfwfp8y9n226l9sy0dfs0sls8dy8f0zquz0y')
  await sienna.transfer(amount, 'secret1xcywp5smmmdxudc7xgnrezt6fnzzvmxqf7ldty')
  const balanceAfter = await sienna.getBalance(agent.address, vk)
  console.info(`SIENNA balance of ${bold(agent.address)}: ${balanceAfter}`)
}
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

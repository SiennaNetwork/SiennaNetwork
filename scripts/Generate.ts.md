# Sienna Scripts: Generate

Generates unsigned transactions for multisig operation.

```typescript
import Fadroma, { bold, timestamp, Console, Deployments } from '@hackbg/fadroma'
const console = new Console('@sienna/scripts/Generate')
```

```typescript
import { ScrtAgentTX, Scrt_1_2 } from '@hackbg/fadroma'

Fadroma.command('amm-pause-v1-init-v2',
  Deployments.activate,
  forMainnet,
  async ({agent, txAgent, deployment, run, cmdArgs})=>{

    const [ newAddress = null ] = cmdArgs

    await txAgent.bundle().wrap(async deployAgent=>{

      const factory_v1 = new API.AMMFactoryClient.v1({
        ...deployment.get('AMM[v1].Factory'),
        agent: deployAgent
      })

      await factory_v1.setStatus(
        "Paused",
        null,
        "Migration to AMMv2 has begun."
      )

      await run(API.upgradeAMMFactory_v1_to_v2, { deployAgent })

    })

  }
)

Fadroma.command('amm-v1-pause',
  Deployments.activate,
  forMainnet,
  async ({agent, txAgent, deployment, run, cmdArgs})=>{
    const [ newAddress = null ] = cmdArgs
    await txAgent.bundle().wrap(async bundle=>{
      const factory = new API.AMMFactoryClient.v1({
        ...deployment.get('AMM[v1].Factory'),
        agent: bundle
      })
      await factory.setStatus(
        "Paused",
        null,
        "Migration to AMMv2 has begun."
      )
    })
  }
)

Fadroma.command('amm-v2-factory',
  Deployments.activate,
  forMainnet,
  async ({agent, txAgent, deployment, run}) => {
    await txAgent.bundle().wrap(async deployAgent=>{
      await run(API.upgradeAMMFactory_v1_to_v2, { deployAgent })
    })
  }
)

/** WARNING: This didn't run as a bundle
  * (probably because of changes to forMainnet) */
Fadroma.command('amm-v2-exchanges-from-v1',
  Deployments.activate,
  forMainnet,
  async ({agent, txAgent, deployment, run}) => {
    await txAgent.bundle().wrap(async deployAgent=>{
      await run(API.cloneAMMExchanges_v1_to_v2, { deployAgent })
    })
  }
)

Fadroma.command('amm-v1-terminate',
  Deployments.activate,
  forMainnet,
  async ({agent, txAgent, deployment, run, cmdArgs})=>{
    const [ newAddress = null ] = cmdArgs
    const factory = new API.AMMFactoryClient.v1({
      ...deployment.get('AMM[v1].Factory'),
      agent: new ScrtAgentTX(agent)
    })
    await factory.setStatus(
      "Migrating",
      newAddress,
      `This contract is terminated. Please migrate to AMM v2 at: ${newAddress}`
    )
  }
)

Fadroma.command('rewards-deploy-v3',
  forMainnet,
  Deployments.activate,
  ({agent, txAgent, deployment, run}) => txAgent.bundle().wrap(deployAgent=>
    run(API.RewardsContract.v2.upgrade.v3, {
      deployAgent,
      template: agent.chain.uploads.load('sienna-rewards@39e87e4.wasm')
    })
  )
)

Fadroma.command('rewards-v3-fix-lp-tokens',
  forMainnet,
  Deployments.activate,
  async ({agent, txAgent, deployment, run}) => txAgent.bundle().wrap(async deployAgent=>{

    const newLPTokenNames =
      Object.keys(deployment.receipts)
        .filter(name=>name.startsWith('AMM[v2].')&&name.endsWith('.LP'))

    console.log({
      newLPTokenNames
    })

    for (const lpTokenName of newLPTokenNames) {
      console.log('checking', lpTokenName)
      const lpTokenReceipt = deployment.receipts[lpTokenName]
      if (!lpTokenReceipt) {
        console.warn('No receipt for', lpTokenName, ' - wtf?!?!?')
        continue
      }
      const rewardPoolName = `${lpTokenName}.Rewards[v3]`
      const rewardPoolReceipt = deployment.receipts[rewardPoolName]
      if (!rewardPoolReceipt) {
        console.info(`No rewards for`, lpTokenName, ' - skipping...')
        continue
      }
      //console.log(123)
      //process.exit(222)
      console.log(
        `Setting staked token for`, rewardPoolName//, 'to', lpTokenReceipt
      )
      //console.log(456)
      delete rewardPoolReceipt.label
      await new API.RewardsClient.v3({...rewardPoolReceipt, agent: deployAgent}).setStakedToken(
        lpTokenReceipt.address,
        lpTokenReceipt.codeHash
      )
    }

  })
)

Fadroma.command('rewards-v3-mainnet',
  forMainnet,
  Deployments.activate,
  ({agent, deployment, run}) => agent.bundle().wrap(deployAgent=>
    run(API.RewardsContract.v2.upgrade.v3, {
      deployAgent,
      template: agent.chain.uploads.load('sienna-rewards@39e87e4.wasm')
    })
  )
)

Fadroma.command('rpt-rewards-v2-to-v3',
  forMainnet,
  Deployments.activate,
  async ({agent, deployment, run, cmdArgs: [ proportion ]})=>{
    proportion = proportion.split(':').map(Number)
    const rpt = new API.RPTClient({ ...deployment.get('RPT'), agent: new ScrtAgentTX(agent) })
    const status = await rpt.status()
  })

Fadroma.command('rewards-v2-close-all',
  forMainnet,
  Deployments.activate,
  async ({agent, deployment, run})=>{
  })

async function forMainnet ({ chain, agent }) {
  if (chain.isMainnet && !!process.env.FADROMA_USE_MULTISIG) {
    const address = process.env.MAINNET_MULTISIG
    if (!address) {
      console.error('Set MAINNET_MULTISIG env var to continue.')
      process.exit(1)
    }
    console.info(bold('Switching to mainnet multisig address:'), address)
    agent = new chain.Agent({ name: 'MAINNET_ADMIN', address, chain })
  }
  const txAgent = new ScrtAgentTX(agent)
  return { agent, txAgent }
}
```

## Entry point

```typescript
Error.stackTraceLimit = Infinity
export default Fadroma.module(import.meta.url)
```

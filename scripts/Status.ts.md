# Sienna Scripts: Status

```typescript
import Fadroma, { bold, timestamp, Console, Deployments } from '@hackbg/fadroma'
const console = new Console('@sienna/scripts/Status')
Fadroma.command('',
  Deployments.activate,
  async function siennaStatus ({
    deployment, agent, cmdArgs: [ vk = 'q1Y3S7Vq8tjdWXCL9dkh' ]
  }) {
    const sienna = new SiennaSnip20Client({ ...deployment.get('SIENNA'), agent })
    try {
      const balance = await sienna.getBalance(agent.address, vk)
      console.info(`SIENNA balance of ${bold(agent.address)}: ${balance}`)
    } catch (e) {
      if (agent.chain.isMainnet) {
        console.error('SIENNA mainnet: no VK')
        return
      }
      const VK = await sienna.setViewingKey(vk)
      console.error(e.message)
    }
  },
  async function mgmtStatus ({
    deployment, agent, MGMT = new MGMTClient({ ...deployment.get('MGMT'), agent })
  }) {
    try {
      const status = await MGMT.status()
      console.debug(`${bold(`MGMT status`)} of ${bold(MGMT.address)}`, status)
    } catch (e) {
      console.error(e.message)
    }
  },
  async function rptStatus ({
    deployment, agent, RPT = new RPTClient({ ...deployment.get('RPT'), agent })
  }) {
    const status = await RPT.status()
    console.info(`RPT status of ${bold(RPT.address)}`)
    console.info(`       Agent: ${bold(agent.address)}`)
    console.debug(status)
  },
  async function ammFactoryStatus_v2 ({
    deployment, agent, factory = new AMMFactoryClient.v2({ ...deployment.get(['AMM[v2].Factory']), agent })
  }) {
    console.info(bold(`Status of AMMv2 Factory at ${factory.address}`))
    console.log()
    const table = []
    for (const exchange of await factory.listExchangesFull()) {
      table.push([exchange.name, exchange.EXCHANGE.address])
    }
    console.table(table)
    console.log()
  },
)
```

```typescript
export async function mgmtProgress ({
  deployment, agent,
  MGMT    = new MGMTClient({ ...deployment.get('MGMT'), agent }),
  address = agent.address,
}: MigrationContext & {
  address: string,
  MGMT: MGMTClient
}) {
  try {
    const progress = await MGMT.progress(address)
    console.info(`${bold(`MGMT progress`)} of ${bold(address)} in ${MGMT.address}`)
    for (const [k,v] of Object.entries(progress)) console.info(' ', bold(k), v)
  } catch (e) {
    console.error(e.message)
  }
}
```

```typescript
export async function printExchanges (EXCHANGES?: any[]) {
  if (!EXCHANGES) {
    console.info('No exchanges found.')
    return
  }
  for (const { name, EXCHANGE, TOKEN_0, TOKEN_1, LP_TOKEN } of EXCHANGES) {
    const { codeId, codeHash, address } = EXCHANGE
    console.info(
      ' ', bold(colors.inverse(name)).padEnd(30), // wat
      `(code id ${bold(String(codeId))})`.padEnd(34), bold(address)
    )
    await print.token(TOKEN_0)
    await print.token(TOKEN_1)
    await print.token(LP_TOKEN)
  }
}
```

```typescript
export function printRewardsContracts (chain: Chain) {
  if (chain && chain.deployments.active) {
    const {name, contracts} = chain.deployments.active
    const isRewardPool = (x: string) => x.startsWith('SiennaRewards_')
    const rewardsContracts = Object.keys(contracts).filter(isRewardPool)
    if (rewardsContracts.length > 0) {
      console.log(`\nRewards contracts in ${bold(name)}:`)
      for (const name of rewardsContracts) {
        console.log(`  ${colors.green('✓')}  ${name}`)
      }
    } else {
      console.log(`\nNo rewards contracts.`)
    }
  } else {
    console.log(`\nSelect a deployment to pick a reward contract.`)
  }
}
```

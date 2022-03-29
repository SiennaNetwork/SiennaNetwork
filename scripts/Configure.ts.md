# Sienna Scripts: Configuration

```typescript
import { Console, bold } from '@hackbg/fadroma'
const console = new Console('@sienna/scripts/Configure')
```

## RPT and LPF accounts

* The **RPT account** (Remaining Pool Tokens) is a special entry 
  in MGMT's vesting schedule; its funds are vested to **the RPT contract's address**,
  and the RPT contract uses them to fund the Reward pools.
  However, the RPT address is only available after deploying the RPT contract,
  which in turn nees MGMT's address, therefore establishing a
  circular dependency. To resolve it, the RPT account in the schedule
  is briefly mutated to point to the deployer's address (before any funds are vested).

```typescript
export function getRPTAccount (schedule: Schedule) {
  return schedule.pools
    .filter((x:any)=>x.name==='MintingPool')[0].accounts
    .filter((x:any)=>x.name==='RPT')[0] }
```

* The **LPF account** (Liquidity Provision Fund) is an entry in MGMT's vesting schedule
  which is vested immediately in full. On devnet and testnet, this can be used
  to provide funding for tester accounts. In practice, testers are funded with an extra
  mint operation in `deployTGE`.

```typescript
export function getLPFAccount (schedule: Schedule) {
  return schedule.pools
    .filter((x:any)=>x.name==='MintingPool')[0].accounts
    .filter((x:any)=>x.name==='LPF')[0] }
```

## Minting testnet tokens

* List of testers that are funded during deployment:

```typescript
export const testers = [
  //"secret1vdf2hz5f2ygy0z7mesntmje8em5u7vxknyeygy",
  "secret13nkfwfp8y9n226l9sy0dfs0sls8dy8f0zquz0y",
  "secret1xcywp5smmmdxudc7xgnrezt6fnzzvmxqf7ldty",
]

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

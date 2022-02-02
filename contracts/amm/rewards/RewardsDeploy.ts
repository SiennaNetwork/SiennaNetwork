import { Console, bold, timestamp } from '@hackbg/fadroma'
const console = Console('@sienna/rewards/Deploy')

import { SiennaSNIP20Contract } from '@sienna/snip20-sienna'
import { adjustRPTConfig } from '@sienna/rpt'
import getSettings, { ONE_SIENNA } from '@sienna/settings'
import { LPTokenContract } from '@sienna/lp-token'
import { RewardsContract } from './RewardsContract'

type MultisigTX = any
const pick       = (...keys) => x => keys.reduce((y, key)=>{y[key]=x[key];return y}, {})
const essentials = pick('codeId', 'codeHash', 'address', 'label')

export async function deployRewards ({
  deployment, agent, run,
  SIENNA  = deployment.getThe('SiennaSNIP20', new SiennaSNIP20Contract({ agent })),
  version = 'v3'
}) {
  const { SSSSS_POOL, RPT_CONFIG_SSSSS } =
    await run(deploySSSSS, { SIENNA, version })
  const { REWARD_POOLS, RPT_CONFIG_SWAP_REWARDS } =
    await run(deployRewardPools, { SIENNA, version })
  return {
    REWARD_POOLS: [ SSSSS_POOL, ...REWARD_POOLS ],
    RPT_CONFIG:   [ ...RPT_CONFIG_SSSSS, ...RPT_CONFIG_SWAP_REWARDS ]
  }
}

Object.assign(deployRewards, {

  /** Deploy legacy Rewards v2. */
  v2: async function deployRewards_v2 ({run}) {
    const { RPT_CONFIG, REWARD_POOLS } = await run(deployRewards, { version: 'v2' })
    return await run(adjustRPTConfig, { RPT_CONFIG })
  },

  /** Deploy latest Rewards v3. */
  v3: async function deployRewards_v3 ({run}) {
    const { RPT_CONFIG, REWARD_POOLS } = await run(deployRewards, { version: 'v3' })
    return await run(adjustRPTConfig, { RPT_CONFIG })
  },

  /** Deploy both versions simultaneously,
    * splitting the balance evenly in the RPT config. */
  v2_and_v3: async function deployRewards_v2_and_v3 ({run}) {
    const [V2, V3] = await Promise.all([
      run(deployRewards, { version: 'v2' }),
      run(deployRewards, { version: 'v3' })
    ])
    const REWARD_POOLS = [ ...V2.REWARD_POOLS, ...V3.REWARD_POOLS ]
    console.table(REWARD_POOLS.reduce(
      (table, {label, address, codeId, codeHash})=>
        Object.assign(table, {
          [label]: { address: address, codeId: codeId, codeHash: codeHash }
        }), {}))
    const RPT_CONFIG  = [ ...V2.RPT_CONFIG,   ...V3.RPT_CONFIG   ]
    return await run(adjustRPTConfig, { RPT_CONFIG })
    return { RPT_CONFIG, REWARD_POOLS }
  }

})

/** Deploy SIENNA/SIENNA SINGLE-SIDED STAKING,
  * (5- or 6-S depending on whether you count the SLASH)
  * a Sienna Rewards pool where you stake SIENNA to earn SIENNA. */
export async function deploySSSSS ({
  run, chain, deployment,
  SIENNA, version,
}) {
  const name = 'SSSSS'
  const lpToken = SIENNA
  const rewardToken = SIENNA
  const { REWARDS: SSSSS_POOL } = await run(deployRewardPool, {
    version, name, lpToken, rewardToken
  })
  return {
    SSSSS_POOL, RPT_CONFIG_SSSSS: [
      [
        SSSSS_POOL.address,
        String(BigInt(getSettings(chain.id).rewardPairs.SIENNA) * ONE_SIENNA)
      ]
    ]
  }
}

/** Deploy the rest of the reward pools,
  * where you stake a LP token to earn SIENNA. */
export async function deployRewardPools ({
  chain, agent, deployment, prefix, run,
  SIENNA  = deployment.getThe('SiennaSNIP20', new SiennaSNIP20Contract({ agent })),
  version = 'v3',
  ammVersion = 'v1',
  split   = 1.0,
}) {
  const REWARDS = new RewardsContract[version]({ prefix, agent })
  await chain.buildAndUpload(agent, [REWARDS])
  const REWARD_POOLS            = []
  const RPT_CONFIG_SWAP_REWARDS = []
  const { swapPairs } = getSettings(chain.id)
  if (!swapPairs || swapPairs.length === 1) {
    throw new Error('@sienna/rewards: needs swapPairs setting')
  }
  const { rewardPairs } = getSettings(chain.id)
  if (!rewardPairs || rewardPairs.length === 1) {
    throw new Error('@sienna/rewards: needs rewardPairs setting')
  }
  for (const name of swapPairs) {
    console.info(bold('Checking if rewards are allocated for'), name)
    if (!rewardPairs[name]) {
      console.info(bold('No rewards for'), name)
      continue
    }
    const exchangeName = `AMM[${ammVersion}].${name}`
    console.info(bold('Need LP token of exchange'), exchangeName)
    const exchange = deployment.receipts[exchangeName]
    if (!exchange) {
      console.error(bold(`Exchange does not exist in deployment`), exchangeName)
      console.error(bold(`Contracts in deployment:`), Object.keys(deployment.receipts).join(' '))
      throw new Error(`@sienna/amm/rewards: Exchange does not exist in deployment: ${exchangeName}`)
    }
    const lpToken = new LPTokenContract({
      address:  exchange.lp_token.address,
      codeHash: exchange.lp_token.code_hash,
      agent
    })
    console.info(bold('Found LP token:'), lpToken.address)
    const { REWARDS } = await run(deployRewardPool, {
      version, name, lpToken, rewardToken: SIENNA,
    })
    REWARD_POOLS.push(REWARDS)
    const reward = BigInt(rewardPairs[name]) / BigInt(1 / split)
    RPT_CONFIG_SWAP_REWARDS.push(
      [REWARDS.address, String(reward * ONE_SIENNA)]
    )
  }
  return { REWARD_POOLS, RPT_CONFIG_SWAP_REWARDS }
}

/** Deploy a single reward pool. Primitive of both deploySSSSS and deployRewardPools */
export async function deployRewardPool ({
  agent, chain, deployment, prefix,
  lpToken,
  rewardToken = deployment.getThe('SiennaSNIP20', new SiennaSNIP20Contract({ agent })),
  name        = 'UNKNOWN',
  version     = 'v3',
}) {
  name = `Rewards[${version}].${name}`
  console.info(bold(`Deploying ${name}:`), version)
  console.info(bold(`Staked token:`), lpToken.address, lpToken.codeHash)
  const REWARDS = new RewardsContract[version]({ lpToken, rewardToken, agent })
  REWARDS.name = name
  await chain.buildAndUpload(agent, [REWARDS])
  await deployment.getOrInit(agent, REWARDS)
  return { REWARDS }
}

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
  SIENNA  = deployment.getThe('SIENNA', new SiennaSNIP20Contract({ agent })),
  version    = 'v3',
  ammVersion = {v3:'v2',v2:'v1'}[version],
}) {
  const { SSSSS_POOL, RPT_CONFIG_SSSSS } =
    await run(deploySSSSS, { SIENNA, version })
  const { REWARD_POOLS, RPT_CONFIG_SWAP_REWARDS } =
    await run(deployRewardPools, { SIENNA, version, ammVersion })
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
  * where you stake SIENNA to earn SIENNA. */
export async function deploySSSSS ({
  run, chain, deployment, agent,
  SIENNA  = deployment.getThe('SIENNA', new SiennaSNIP20Contract({ agent })),
  version = 'v3',
  settings: { rewardPairs } = getSettings(chain.id),
}) {
  if (!rewardPairs || rewardPairs.length === 1) {
    throw new Error(`@sienna/rewards: needs rewardPairs setting for ${chain.id}`)
  }
  const name        = 'SSSSS'
  const lpToken     = SIENNA
  const rewardToken = SIENNA
  const { REWARDS } = await run(deployRewardPool, { version, name, lpToken: SIENNA })
  return {
    SSSSS_POOL: REWARDS, RPT_CONFIG_SSSSS: [
      [
        REWARDS.address,
        String(BigInt(getSettings(chain.id).rewardPairs.SIENNA) * ONE_SIENNA)
      ]
    ]
  }
}

/** Deploy the rest of the reward pools, where you stake a LP token to earn SIENNA. */
export async function deployRewardPools ({
  chain, agent, deployment, prefix, run,
  SIENNA                    = deployment.getThe('SIENNA', new SiennaSNIP20Contract({ agent })),
  version                   = 'v3',
  ammVersion                = {v3:'v2',v2:'v1'}[version],
  settings: { rewardPairs } = getSettings(chain.id),
  REWARD_POOLS              = [],
  split                     = 1.0,
  RPT_CONFIG_SWAP_REWARDS   = [],
}) {
  if (!rewardPairs || rewardPairs.length === 1) {
    throw new Error(`@sienna/rewards: needs rewardPairs setting for ${chain.id}`)
  }
  for (let [name, reward] of Object.entries(rewardPairs)) {
    // ignore SSSSS pool - that is deployed separately
    if (name === 'SIENNA') continue
    // find LP token to attach to
    const lpTokenName = `AMM[${ammVersion}].${name}.LP`
    const lpToken = deployment.getThe(lpTokenName, new LPTokenContract({ agent }))
    // create a reward pool
    const options = { version, name, lpToken }
  console.info('Deploying', bold(name), version, 'for', bold(lpTokenName))
    const { REWARDS } = await run(deployRewardPool, options)
    REWARD_POOLS.push(REWARDS)
    // collect the RPT budget line
    const reward = BigInt(rewardPairs[name]) / BigInt(1 / split)
    const budget = [REWARDS.address, String(reward * ONE_SIENNA)]
    RPT_CONFIG_SWAP_REWARDS.push(budget)
  }
  return { REWARD_POOLS, RPT_CONFIG_SWAP_REWARDS }
}

/** Deploy a single reward pool. Primitive of both deploySSSSS and deployRewardPools */
export async function deployRewardPool ({
  agent, chain, deployment, prefix,
  lpToken,
  rewardToken = deployment.getThe('SIENNA', new SiennaSNIP20Contract({ agent })),
  name        = 'UNKNOWN',
  version     = 'v3',
}) {
  name = `Rewards[${version}].${name}`
  console.info(bold(`Staked token:`))
  console.info(' ', lpToken.address)
  console.info(' ', lpToken.codeHash)
  const REWARDS = new RewardsContract[version]({ lpToken, rewardToken, agent })
  await chain.buildAndUpload(agent, [REWARDS])
  REWARDS.name = name
  await chain.buildAndUpload(agent, [REWARDS])
  await deployment.getOrInit(agent, REWARDS)
  return { REWARDS }
}

import { Console, bold, timestamp } from '@hackbg/fadroma'

const console = Console('@sienna/rewards/Upgrade')

import { SiennaSNIP20Contract } from '@sienna/snip20-sienna'

import { RewardsContract } from './RewardsContract'
import { deployRewardPool } from './RewardsDeploy'

export async function upgradeRewards ({
  timestamp, chain, agent, deployment, prefix, run,

  OldRewardsContract,
  NewRewardsContract,

  SIENNA = deployment.getThe('SiennaSNIP20', new SiennaSNIP20Contract({agent})),
  RPT    = deployment.getThe('SiennaRPT',    new SiennaSNIP20Contract({agent})),
  REWARD_POOLS = deployment.getAll('SiennaRewards_v2', name => new OldRewardsContract({agent})),

  version,
  suffix = `+${timestamp}`
}) {
  console.log({REWARD_POOLS})
  const NEW_REWARD_POOLS: RewardsContract[] = []
  for (const REWARDS of REWARD_POOLS) {
    const LP_TOKEN = await REWARDS.lpToken()
    const {symbol} = await LP_TOKEN.info
    let name
    if (symbol === 'SIENNA') {
      name = 'SSSSS'
    } else {
      const [LP, TOKEN0, TOKEN1] = (await LP_TOKEN.friendlyName).split('-')
      name = `${TOKEN0}-${TOKEN1}`
    }
    console.log()
    console.info(bold('Upgrading reward pool'), name)
    NEW_REWARD_POOLS.push((await run(deployRewardPool, {
      version, name, suffix,
      lpToken: LP_TOKEN, rewardToken: SIENNA
    })).REWARDS)
  }
  console.info(`Deployed`, bold(String(NEW_REWARD_POOLS.length)), version, `reward pools.`)
  return { REWARD_POOLS: NEW_REWARD_POOLS }
}

/** Create v3 pools corresponding to the v2 pools. */
upgradeRewards.v2_to_v3 = function upgradeRewards_v2_to_v3 (input) {
  return upgradeRewards({
    ...input,
    OldRewardsContract: RewardsContract.v2,
    NewRewardsContract: RewardsContract.v3,
    version: 'v3'
  })
}

/** Test migration procedure. */
upgradeRewards.v3_to_v3 = function upgradeRewards_v3_to_v3 (input) {
  return upgradeRewards({
    ...input,
    OldRewardsContract: RewardsContract.v3,
    NewRewardsContract: RewardsContract.v3,
    version: 'v3'
  })
}

upgradeRewards.adjustBalances = function upgradeRewards_adjustBalances (input) {
  throw 'TODO'
}

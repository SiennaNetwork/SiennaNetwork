import { IAgent, IChain, Deployment, timestamp } from '@fadroma/scrt'
import type { SNIP20Contract } from '@fadroma/snip20'
import { RewardsContract } from '@sienna/api'

export async function deployRewardPool ({
  chain,
  admin,
  deployment,
  REWARDS,
  suffix,
  lpToken,
  rewardToken,
}: {
  chain:       IChain
  admin:       IAgent
  deployment:  Deployment
  REWARDS:     RewardsContract
  suffix:      string
  lpToken:     SNIP20Contract
  rewardToken: SNIP20Contract
}) {
  const {codeId, codeHash} = REWARDS
  const options = {
    codeId, codeHash,
    prefix: deployment.name, suffix: `_${suffix}+${timestamp()}`,
    admin, instantiator: admin,
    chain, lpToken, rewardToken
  }
  const rewardPool = new RewardsContract(options)
  const receipt    = deployment.contracts[rewardPool.label]
  await rewardPool.instantiateOrExisting(receipt)
  return rewardPool
}

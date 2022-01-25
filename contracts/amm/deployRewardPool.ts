import { Migration, Console } from '@hackbg/fadroma'

import type { SNIP20Contract } from '@fadroma/snip20'
import { RewardsContract } from '@sienna/api'
import { workspace } from '@sienna/settings'

const console = Console('@sienna/amm/deployRewardPool')

export async function deployRewardPool (options: Migration & {
  apiVersion?:  'v2'|'v3'
  suffix?:      string
  lpToken?:     SNIP20Contract
  rewardToken?: SNIP20Contract
}) {

  const {
    timestamp,
    admin,
    deployment,
    prefix,
  
    lpToken,
    rewardToken,
    apiVersion  = 'v3',
  } = options

  const tokenInfo = await lpToken.q(admin).tokenInfo()
  const suffix    = `_${tokenInfo.symbol}_${apiVersion}+${timestamp}`

  const contract = new RewardsContract({
    workspace,
    instantiator: admin,
    prefix,
    name: 'SiennaRewards',
    suffix,
    lpToken,
    rewardToken
  })

  await contract.buildInDocker()
  await contract.uploadAs(admin)

  if (apiVersion === 'v2') {
    // override init msg for legacy api
    const initMsg = {
      admin:        admin.address,
      lp_token:     lpToken.link,
      reward_token: rewardToken.link,
      viewing_key:  "",
      ratio:        ["1", "1"],
      threshold:    15940,
      cooldown:     15940,
    }
    // use Object.assign to avoid type check
    Object.assign(contract, { initMsg })
  }

  const receipt = deployment.contracts[contract.label]
  await contract.instantiateOrExisting(receipt)

  return contract

}

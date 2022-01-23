import { Migration } from '@fadroma/scrt'
import type { SNIP20Contract } from '@fadroma/snip20'
import { RewardsContract } from '@sienna/api'

export async function deployRewardPool (options: Migration & {
  apiVersion?:   'v2'|'v3'
  suffix?:       string
  lpToken?:      SNIP20Contract
  rewardToken?:  SNIP20Contract
}) {

  const {

    timestamp,

    chain,
    admin,
    contracts,
  
    apiVersion  = 'v3',
    suffix      = `_${apiVersion}+${timestamp}`,
    lpToken,
    rewardToken

  } = options

  const contract = new RewardsContract()
  await chain.buildAndUpload(contract)

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

  const receipt = contracts[contract.label]
  await contract.instantiateOrExisting(receipt)

  return contract

}

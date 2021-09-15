import type { Agent } from '@fadroma/ops'

// TGE /////////////////////////////////////////////////////////////////////////////////////////////

export { SiennaSNIP20 } from '../api/SNIP20'
export { MGMT } from '../api/MGMT'
export { RPT } from '../api/RPT'

// Swap ////////////////////////////////////////////////////////////////////////////////////////////

export { Factory as AMMFactory } from '../api/Factory'
export { AMM as AMMExchange } from '../api/AMM'
export { AMMSNIP20 } from '../api/SNIP20'

// Rewards /////////////////////////////////////////////////////////////////////////////////////////

import { Rewards as RewardPool } from '../api/Rewards'
export { RewardPool }

import { LPToken } from '../api/SNIP20'
export { LPToken }

export function rewardPools (agent: Agent, pairs: Array<string>) {
  const pools = {}
  for (const pair of pairs) {
    pools[`LP_${pair}`] = new LPToken(agent, pair)
    pools[`RP_${pair}`] = new RewardPool(agent, pair) }
  return pools }

// IDO /////////////////////////////////////////////////////////////////////////////////////////////

export { IDO } from '../api/IDO'

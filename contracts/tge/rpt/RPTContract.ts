import { Agent, Scrt_1_2 } from '@hackbg/fadroma'
import type { SNIP20Contract_1_2 } from '@fadroma/snip20'

import type { MGMTContract } from '@sienna/mgmt'

import type { LinearMapAnd_Uint128 as LinearMap, Uint128 } from './schema/init'
import { RPTTransactions } from './RPTTransactions'
import { RPTQueries }      from './RPTQueries'

export type RPTRecipient = string
export type RPTAmount    = string
export type RPTConfig    = [RPTRecipient, RPTAmount][]

export class RPTContract extends Scrt_1_2.Contract<RPTTransactions, RPTQueries> {
  crate = 'sienna-rpt'
  name  = 'SiennaRPT'

  Transactions = RPTTransactions
  Queries      = RPTQueries

  /** query contract status */
  get status() {
    return this.q().status().then(({status})=>status)
  }

}

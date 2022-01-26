import { Agent, ContractState, Scrt_1_2 } from '@hackbg/fadroma'
import type { SNIP20Contract_1_2 } from '@fadroma/snip20'

import type { MGMTContract } from '@sienna/mgmt'
import { workspace } from '@sienna/settings'

import type { LinearMapAnd_Uint128 as LinearMap, Uint128 } from './schema/init'
import { RPTTransactions } from './RPTTransactions'
import { RPTQueries }      from './RPTQueries'

export class RPTContract extends Scrt_1_2.Contract<RPTTransactions, RPTQueries> {

  workspace = workspace
  crate     = 'sienna-rpt'
  name      = 'SiennaRPT'

  Transactions = RPTTransactions
  Queries      = RPTQueries

  constructor (options: ContractState & {
    admin?:   Agent,
    config?:  LinearMap
    portion?: Uint128
    SIENNA?:  SNIP20Contract_1_2
    MGMT?:    MGMTContract
  } = {}) {

    super(options)

    Object.assign(this.initMsg, {
      token:   options?.SIENNA?.linkPair,
      mgmt:    options?.MGMT?.linkPair,
      portion: options.portion,
      config:  [[options.admin?.address, options.portion]]
    })

    Object.defineProperties(this.initMsg, {
      token: { enumerable: true, get () { return options?.SIENNA?.linkPair } },
      mgmt:  { enumerable: true, get () { return options?.MGMT?.linkPair   } }
    })

  }

  /** query contract status */
  get status() {
    return this.q().status().then(({status})=>status)
  }

}

export type RPTRecipient = string
export type RPTAmount    = string
export type RPTConfig    = [RPTRecipient, RPTAmount][]

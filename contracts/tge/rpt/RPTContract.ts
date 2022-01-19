import type { IAgent, ContractState } from '@fadroma/scrt'
import type { SNIP20Contract_1_0 } from '@fadroma/snip20'
import { AugmentedScrtContract_1_0, TransactionExecutor, QueryExecutor } from "@fadroma/scrt"

import type { MGMTContract } from '@sienna/mgmt'
import { workspace } from '@sienna/settings'

import type { LinearMapFor_HumanAddrAnd_Uint128, Uint128 } from './rpt/init'

import { RPTTransactions } from './RPTTransactions'
import { RPTQueries }      from './RPTQueries'
export class RPTContract extends AugmentedScrtContract_1_0<RPTTransactions, RPTQueries> {

  crate = 'sienna-rpt'

  name = 'SiennaRPT'

  Transactions = RPTTransactions

  Queries      = RPTQueries

  constructor (options: ContractState & {
    admin?:   IAgent,
    config?:  LinearMapFor_HumanAddrAnd_Uint128
    portion?: Uint128
    SIENNA?:  SNIP20Contract_1_0
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

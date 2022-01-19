import type { IAgent, ContractState } from '@fadroma/scrt'
import type { SNIP20Contract } from '@fadroma/snip20'
import { AugmentedScrtContract_1_2, TransactionExecutor, QueryExecutor } from "@fadroma/scrt"
import { workspace } from '@sienna/settings'
import type { Schedule } from './schema/init'

import { MGMTTransactions } from './MGMTTransactions'
import { MGMTQueries }      from './MGMTQueries'
export class MGMTContract extends AugmentedScrtContract_1_2<MGMTTransactions, MGMTQueries> {

  workspace = workspace
  crate     = 'sienna-mgmt'
  name      = 'SiennaMGMT'

  Transactions = MGMTTransactions
  Queries      = MGMTQueries

  constructor (options: ContractState & {
    admin?:    IAgent,
    schedule?: Schedule,
    SIENNA?:   SNIP20Contract
  } = {}) {

    super(options)

    if (options.admin) {
      this.uploader      = options.admin
      this.instantiator  = options.admin
      this.initMsg.admin = options.admin.address
    }

    if (options.schedule) {
      this.schedule = options.schedule
    }

    // auto get token address after it's deployed
    Object.defineProperty(this.initMsg, 'token', {
      enumerable: true,
      get () { return options.SIENNA.linkPair }
    })

  }

  /** query current schedule */
  get schedule (): Promise<Schedule> {
    if (this.address) {
      return this.q().schedule()
    } else {
      return this.initMsg.schedule
    }
  }

  set schedule (schedule: Schedule|Promise<Schedule>) {
    if (this.address) {
      throw new Error('Use the configure method to set the schedule of a deployed contract.')
    } else {
      Promise.resolve(schedule).then(schedule=>this.initMsg.schedule = schedule)
    }
  }

}

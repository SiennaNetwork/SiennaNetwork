import { Agent, ContractState, Scrt_1_0 } from '@hackbg/fadroma'
import type { SNIP20Contract } from '@fadroma/snip20'

import { workspace } from '@sienna/settings'
import type { InitMsg, Schedule } from './schema/init'
import { MGMTTransactions } from './MGMTTransactions'
import { MGMTQueries }      from './MGMTQueries'

export class MGMTContract extends Scrt_1_0.Contract<
  MGMTTransactions,
  MGMTQueries
> {

  workspace = workspace
  crate     = 'sienna-mgmt'
  name      = 'SiennaMGMT'

  initMsg?: InitMsg

  Transactions = MGMTTransactions
  Queries      = MGMTQueries

  constructor (options: ContractState & {
    admin?:    Agent,
    schedule?: Schedule,
    SIENNA?:   SNIP20Contract
  } = {}) {

    super(options)

    if (options.admin) {
      this.uploader = options.admin
      this.creator  = options.admin
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

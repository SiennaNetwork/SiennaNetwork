import { Agent, Scrt_1_2 } from '@hackbg/fadroma'
import type { SNIP20Contract } from '@fadroma/snip20'

import type { Init, Schedule } from './schema/init'
import { MGMTTransactions } from './MGMTTransactions'
import { MGMTQueries }      from './MGMTQueries'

export class MGMTContract extends Scrt_1_2.Contract<
  MGMTTransactions,
  MGMTQueries
> {
  crate = 'sienna-mgmt'
  name  = 'SiennaMGMT'
  initMsg?: Init
  Transactions = MGMTTransactions
  Queries = MGMTQueries

  /** query current schedule */
  get schedule (): Promise<Schedule> {
    if (this.address) {
      return this.q().schedule()
    } else {
      return Promise.resolve(this.initMsg.schedule)
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

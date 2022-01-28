import { Scrt_1_2 } from '@hackbg/fadroma'
import type { SNIP20Contract } from '@fadroma/snip20'

export class MGMTQueries extends Scrt_1_2.Contract.Queries {

  /** query contgract status */
  status () {
    const msg = { schedule: {} }
    return this.query(msg)
  }

  /** see how much is claimable by someone at a certain time */
  schedule () {
    const msg = { schedule: {} }
    return this.query(msg)
  }

  /** see how much is claimable by someone at a certain time */
  progress (address: any, time = +new Date()) {
    time = Math.floor(time / 1000) // JS msec -> CosmWasm seconds
    const msg = { address, time }
    return this.query(msg)
  }

}

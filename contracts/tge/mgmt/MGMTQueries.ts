import { QueryExecutor } from '@fadroma/scrt'

export class MGMTQueries extends QueryExecutor {

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

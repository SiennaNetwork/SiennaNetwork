import { Scrt_1_2 } from '@hackbg/fadroma'

export class MGMTQueries extends Scrt_1_2.Contract.Queries {

  /** Query contract status */
  status () {
    return this.query({ status: {} })
  }

  /** See the full schedule */
  schedule () {
    return this.query({ schedule: {} })
  }

  /** Check how much is claimable by someone at a certain time */
  async progress (address: any, time = +new Date()): Promise<{
    time:     number
    launcher: number
    elapsed:  number
    unlocked: string
    claimed:  string
  }> {
    time = Math.floor(time / 1000) // JS msec -> CosmWasm seconds
    const msg = { progress: { address, time } }
    const { progress } = await this.query(msg)
    return progress
  }

}

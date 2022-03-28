import { Client, Snip20Client } from '@hackbg/fadroma'

export class MGMTClient extends Client {

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
    const { progress } = await this.query({ progress: { address, time } })
    return progress
  }

  /** take over a SNIP20 token */
  async acquire (token: Snip20Client) {
    const tx1 = await token.setMinters([this.address])
    const tx2 = await token.changeAdmin(this.address)
    return [tx1, tx2]
  }

  /** load a schedule */
  async configure (schedule: any) {
    return this.execute({ configure: { schedule } })
  }

  /** launch the vesting */
  launch () {
    return this.execute({ launch: {} })
  }

  /** claim accumulated portions */
  claim (claimant: any) {
    return this.execute({ claim: {} })
  }

  /** add a new account to a pool */
  add (pool_name: any, account: any) {
    return this.execute({ add_account: { pool_name, account } })
  }

  /** set the admin */
  setOwner (new_admin: any) {
    return this.execute({ set_owner: { new_admin } })
  }

}

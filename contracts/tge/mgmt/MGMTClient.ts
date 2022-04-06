import { Client, Snip20Client } from '@hackbg/fadroma'


type Link = { address: string, code_hash: string }


export abstract class MGMTClient extends Client {

  static "legacy" = class MGMTClient_TGE extends MGMTClient {

    /** Query contract status */
    status() {
      return this.query({ status: {} })
    }

    /** See the full schedule */
    schedule() {
      return this.query({ schedule: {} })
    }

    /** Check how much is claimable by someone at a certain time */
    async progress(address: any, time = +new Date()): Promise<{
      time: number
      launcher: number
      elapsed: number
      unlocked: string
      claimed: string
    }> {
      time = Math.floor(time / 1000) // JS msec -> CosmWasm seconds
      const { progress } = await this.query({ progress: { address, time } })
      return progress
    }

    /** take over a SNIP20 token */
    async acquire(token: Snip20Client) {
      const tx1 = await token.setMinters([this.address])
      const tx2 = await token.changeAdmin(this.address)
      return [tx1, tx2]
    }

    /** load a schedule */
    async configure(schedule: any) {
      return this.execute({ configure: { schedule } })
    }

    /** launch the vesting */
    launch() {
      return this.execute({ launch: {} })
    }

    /** claim accumulated portions */
    claim(claimant: any) {
      return this.execute({ claim: {} })
    }

    /** add a new account to a pool */
    add(pool_name: any, account: any) {
      return this.execute({ add_account: { pool_name, account } })
    }

    /** set the admin */
    setOwner(new_admin: any) {
      return this.execute({ set_owner: { new_admin } })
    }
  }

  static "vested" = class MGMTClient_Vested extends MGMTClient {
    /** load a schedule */
    async configure(schedule: any) {
      return this.execute({ configure: { schedule } })
    }
    /** launch the vesting */
    launch() {
      return this.execute({ launch: {} })
    }

    /** claim accumulated portions */
    claim() {
      return this.execute({ claim: {} })
    }

    /** add a new account to a pool */
    add(pool_name: any, account: any) {
      return this.execute({ add_account: { pool_name, account } })
    }

    /** Change the admin of the contract, requires the other user to accept */
    change_admin(new_admin: any) {
      return this.execute({ auth: { change_admin: { address: new_admin } } })
    }

    /** accept becoming an admin */
    accept_admin() {
      return this.execute({ auth: { accept_admin: {} } })
    }

    /** See the full schedule */
    schedule() {
      return this.query({ schedule: {} })
    }

    history(start: number, limit: number) {
      return this.query({ history: { start, limit } })
    }

    config() {
      return this.query({ config: {} })
    }

    async progress(address: any, time = +new Date()): Promise<{
      time: number
      launcher: number
      elapsed: number
      unlocked: string
      claimed: string
    }> {
      time = Math.floor(time / 1000) // JS msec -> CosmWasm seconds
      const { progress } = await this.query({ progress: { address, time } })
      return progress
    }

  }

}

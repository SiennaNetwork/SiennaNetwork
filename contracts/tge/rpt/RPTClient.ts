import { Client, Snip20Client } from '@hackbg/fadroma'

export class RPTClient extends Client {

  /** query contract status */
  async status () {
    return (await this.query({ status: {} })).status
  }

  /** set the vesting recipients */
  configure (config = []) {
    return this.execute({ configure: { config } })
  }

  /** claim from mgmt and distribute to recipients */
  vest () {
    return this.execute({ vest: {} })
  }

  /** change the admin */
  setOwner (new_admin) {
    return this.execute({ set_owner: { new_admin } })
  }

}

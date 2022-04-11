import { Client, Snip20Client } from '@hackbg/fadroma'

export type RPTRecipient = string
export type RPTAmount = string
export type RPTConfig = [RPTRecipient, RPTAmount][]

export abstract class RPTClient extends Client {

  static "legacy" = class RPTClient_TGE extends RPTClient {
    /** query contract status */
    async status() {
      return (await this.query({ status: {} })).status
    }

    /** set the vesting recipients */
    configure(config = []) {
      return this.execute({ configure: { config } })
    }

    /** claim from mgmt and distribute to recipients */
    vest() {
      return this.execute({ vest: {} })
    }

    /** change the admin */
    setOwner(new_admin) {
      return this.execute({ set_owner: { new_admin } })
    }
  }
  static "vested" = class RPTClient_Vested extends RPTClient {
    configuration() {
      return this.query({ configuration: {} });
    }

    set_distribution(distribution) {
      return this.execute({ set_distribution: { distribution }});
    }

    vest() {
      return this.execute({ vest: {} });
    }
    
  }
}


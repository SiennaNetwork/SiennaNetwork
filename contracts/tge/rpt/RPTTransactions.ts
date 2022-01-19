import { TransactionExecutor } from '@fadroma/scrt'

export class RPTTransactions extends TransactionExecutor {

  /** set the splitt proportions */
  configure (config = []) {
    const msg = { configure: { config } }
    return this.agent.execute(this.contract, msg)
  }

  /** claim portions from mgmt and distribute them to recipients */
  vest () {
    const msg = { vest: {} }
    return this.agent.execute(this.contract, msg)
  }

  /** set the admin */
  setOwner (new_admin) {
    const msg = { set_owner: { new_admin } }
    return this.agent.execute(this.contract, msg)
  }

}

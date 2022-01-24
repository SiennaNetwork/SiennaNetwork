import { TransactionExecutor } from '@hackbg/fadroma'

export class RPTTransactions extends TransactionExecutor {

  /** set the splitt proportions */
  configure (config = []) {
    const msg = { configure: { config } }
    return this.execute(msg)
  }

  /** claim portions from mgmt and distribute them to recipients */
  vest () {
    const msg = { vest: {} }
    return this.execute(msg)
  }

  /** set the admin */
  setOwner (new_admin) {
    const msg = { set_owner: { new_admin } }
    return this.execute(msg)
  }

}

import type { SNIP20Contract } from '@fadroma/snip20'
import { TransactionExecutor } from '@fadroma/scrt'

export class MGMTTransactions extends TransactionExecutor {

  /** take over a SNIP20 token */
  async acquire (snip20: SNIP20Contract) {
    const tx1 = await snip20.tx(this.agent).setMinters([this.contract.address]);
    const tx2 = await snip20.tx(this.agent).changeAdmin(this.contract.address);
    return [tx1, tx2]
  }

  /** load a schedule */
  async configure (schedule: any) {
    const msg = { configure: { schedule } }
    return this.agent.execute(this.contract, msg)
  }

  /** launch the vesting */
  launch () {
    const msg = { launch: {} }
    return this.agent.execute(this.contract, msg)
  }

  /** claim accumulated portions */
  claim (claimant: any) {
    const msg = { claim: {} }
    return this.agent.execute(this.contract, msg)
  }

  /** add a new account to a pool */
  add (pool_name: any, account: any) {
    const msg = { add_account: { pool_name, account } }
    return this.agent.execute(this.contract, msg)
  }

  /** set the admin */
  setOwner (new_admin: any) {
    const msg = { set_owner: { new_admin } }
    return this.agent.execute(this.contract, msg)
  }

}

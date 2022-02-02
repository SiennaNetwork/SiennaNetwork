import { Scrt_1_2,  SNIP20Contract } from '@hackbg/fadroma'

export class MGMTTransactions extends Scrt_1_2.Contract.Transactions {

  /** take over a SNIP20 token */
  async acquire (snip20: SNIP20Contract) {
    const tx1 = await snip20.tx(this.agent).setMinters([this.contract.address]);
    const tx2 = await snip20.tx(this.agent).changeAdmin(this.contract.address);
    return [tx1, tx2]
  }

  /** load a schedule */
  async configure (schedule: any) {
    const msg = { configure: { schedule } }
    return this.execute(msg)
  }

  /** launch the vesting */
  launch () {
    const msg = { launch: {} }
    return this.execute(msg)
  }

  /** claim accumulated portions */
  claim (claimant: any) {
    const msg = { claim: {} }
    return this.execute(msg)
  }

  /** add a new account to a pool */
  add (pool_name: any, account: any) {
    const msg = { add_account: { pool_name, account } }
    return this.execute(msg)
  }

  /** set the admin */
  setOwner (new_admin: any) {
    const msg = { set_owner: { new_admin } }
    return this.execute(msg)
  }

}

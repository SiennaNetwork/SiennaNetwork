import type { Agent } from '@fadroma/ops'
import type { ContractAPIOptions } from "@fadroma/scrt"
import type { ScheduleFor_HumanAddr } from './mgmt/init'
import type { SNIP20 } from './SNIP20'
import { ScrtContract, loadSchemas } from "@fadroma/scrt"
import { abs } from '../ops/index'

const schema = loadSchemas(import.meta.url, {
  initMsg:     "./mgmt/init.json",
  queryMsg:    "./mgmt/query.json",
  queryAnswer: "./mgmt/response.json",
  handleMsg:   "./mgmt/handle.json",
})

export class MGMT extends ScrtContract {

  constructor (options: {
    admin:    Agent,
    schedule: ScheduleFor_HumanAddr,
    SIENNA:   SNIP20
  }) {
    super({ agent: options.admin, schema })
    Object.assign(this.init.msg, {
      admin:    options.admin.address,
      token:    options.SIENNA.linkPair,
      schedule: options.schedule,
    })
  }

  code = { ...this.code, workspace: abs(), crate: 'sienna-mgmt' }

  init = { ...this.init, label: 'SiennaMGMT', msg: {} }

  /** query contract status */
  get status() {
    return this.q.status({});
  }

  /** query current schedule */
  get schedule() {
    return this.q.schedule({});
  }

  /** take over a SNIP20 token */
  acquire = async (snip20: any) => {
    const tx1 = await snip20.setMinters([this.address]);
    const tx2 = await snip20.changeAdmin(this.address);
    return [tx1, tx2];
  };

  /** load a schedule */
  configure = (schedule: any) => this.tx.configure({ schedule });

  /** launch the vesting */
  launch = () => this.tx.launch({});

  /** claim accumulated portions */
  claim = (claimant: any) => this.tx.claim({});

  /** see how much is claimable by someone at a certain time */
  progress = (address: any, time = +new Date()) =>
    this.q.progress({
      address,
      time: Math.floor(time / 1000) /* JS msec -> CosmWasm seconds */,
    });

  /** add a new account to a pool */
  add = (pool_name: any, account: any) => this.tx.add_account({ pool_name, account });

  /** set the admin */
  setOwner = (new_admin: any) => this.tx.set_owner({ new_admin });
}

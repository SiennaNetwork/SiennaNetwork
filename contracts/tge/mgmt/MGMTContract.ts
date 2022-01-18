import type { IAgent, ContractState } from '@fadroma/scrt'
import type { SNIP20Contract } from '@fadroma/snip20'
import { ScrtContract_1_2 } from "@fadroma/scrt"
import { workspace } from '@sienna/settings'
import type { Schedule } from './schema/init'

export class MGMTContract extends ScrtContract_1_2 {

  crate = 'sienna-mgmt'

  name  = 'SiennaMGMT'

  constructor (options: ContractState & {
    admin?:    IAgent,
    schedule?: Schedule,
    SIENNA?:   SNIP20Contract
  } = {}) {

    super(options)

    if (options.admin) {
      this.uploader      = options.admin
      this.instantiator  = options.admin
      this.initMsg.admin = options.admin.address
    }

    if (options.schedule) {
      this.schedule = options.schedule
    }

    // auto get token address after it's deployed
    Object.defineProperty(this.initMsg, 'token', {
      enumerable: true,
      get () { return options.SIENNA.linkPair }
    })

  }

  /** query current schedule */
  get schedule (): Promise<Schedule> {
    if (this.address) {
      return this.q.schedule({})
    } else {
      return this.initMsg.schedule
    }
  }
  set schedule (schedule: Schedule|Promise<Schedule>) {
    if (this.address) {
      throw new Error('Use the configure method to set the schedule of a deployed contract.')
    }
    Promise.resolve(schedule).then(schedule=>this.initMsg.schedule = schedule)
  }

  /** query contract status */
  get status () {
    return this.q.status({})
  }

  /** take over a SNIP20 token */
  acquire = async (snip20: any) => {
    const tx1 = await snip20.setMinters([this.address]);
    const tx2 = await snip20.changeAdmin(this.address);
    return [tx1, tx2]
  }

  /** load a schedule */
  configure = (schedule: any) => this.tx.configure({ schedule })

  /** launch the vesting */
  launch = () => this.tx.launch({})

  /** claim accumulated portions */
  claim = (claimant: any) => this.tx.claim({})

  /** see how much is claimable by someone at a certain time */
  progress = (address: any, time = +new Date()) =>
    this.q.progress({
      address,
      time: Math.floor(time / 1000/* JS msec -> CosmWasm seconds */) })

  /** add a new account to a pool */
  add = (pool_name: any, account: any) => this.tx.add_account({ pool_name, account })

  /** set the admin */
  setOwner = (new_admin: any) => this.tx.set_owner({ new_admin })
}

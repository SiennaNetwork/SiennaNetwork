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
      return this.q().schedule()
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

  tx (agent: IAgent = this.instantiator): MGMTContractExecutor {
    return new MGMTContractExecutor(this, agent)
  }

  q (agent: IAgent = this.instantiator): MGMTContractQuerier {
    return new MGMTContractQuerier(this, agent)
  }

}

export class MGMTContractExecutor {

  constructor (
    readonly contract: MGMTContract,
    readonly agent:    IAgent
  ) {}

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

export class MGMTContractQuerier {

  constructor (
    readonly contract: MGMTContract,
    readonly agent:    IAgent
  ) {}

  /** query contgract status */
  status () {
    const msg = { schedule: {} }
    return this.agent.query(this.contract, msg)
  }

  /** see how much is claimable by someone at a certain time */
  schedule () {
    const msg = { schedule: {} }
    return this.agent.query(this.contract, msg)
  }

  /** see how much is claimable by someone at a certain time */
  progress (address: any, time = +new Date()) {
    time = Math.floor(time / 1000) // JS msec -> CosmWasm seconds
    const msg = { address, time }
    return this.agent.query(this.contract, msg)
  }

}

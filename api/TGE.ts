import type { Agent } from '@fadroma/ops'
import type { LinearMapFor_HumanAddrAnd_Uint128, Uint128 } from './rpt/init'
import type { ScheduleFor_HumanAddr } from './mgmt/init'
import { SNIP20 } from './SNIP20'
import { ScrtContract, loadSchemas } from "@fadroma/scrt"
import { abs } from '../ops/index'
import { randomHex, timestamp } from '@fadroma/tools'

export class SiennaSNIP20 extends SNIP20 {

  code = {
    ...this.code, workspace: abs(), crate: "snip20-sienna" }

  init = {
    ...this.init,
    label: this.init.label || `SiennaSNIP20@${timestamp()}`,
    msg: { name: "Sienna",
           symbol: "SIENNA",
           decimals: 18,
           config: { public_total_supply: true } } }

  constructor (options: {
    prefix?: string,
    admin?: Agent
  } = {}) {
    super({
      prefix: options?.prefix,
      agent:  options?.admin
    })
    this.init.msg.prng_seed = randomHex(36)
  }

  static attach = (
    address:  string,
    codeHash: string,
    agent:    Agent
  ) => {
    const instance = new SiennaSNIP20({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }

}

export class MGMTContract extends ScrtContract {
  static schema = loadSchemas(import.meta.url, {
    initMsg:     "./mgmt/init.json",
    queryMsg:    "./mgmt/query.json",
    queryAnswer: "./mgmt/response.json",
    handleMsg:   "./mgmt/handle.json"
  })
  static attach = (address: string, codeHash: string, admin: Agent) => {
    const contract = new MGMTContract({ admin })
    contract.init.agent    = admin
    contract.init.address  = address
    contract.blob.codeHash = codeHash
  }
  code = { ...this.code, workspace: abs(), crate: 'sienna-mgmt' }
  init = { ...this.init, label: 'SiennaMGMT', msg: {} }
  constructor (options: {
    prefix?:   string
    admin?:    Agent
    schedule?: ScheduleFor_HumanAddr
    SIENNA?:   SNIP20
  } = {}) {
    super({
      prefix: options?.prefix,
      agent: options?.admin,
      schema: MGMTContract.schema
    })
    Object.assign(this.init.msg, {
      admin:    options.admin?.address,
      schedule: options.schedule,
    })
    // auto get token address after it's deployed
    Object.defineProperty(this.init.msg, 'token', {
      enumerable: true,
      get () { return options.SIENNA.linkPair }
    })
  }

  /** query contract status */
  get status() { return this.q.status({}) }
  /** query current schedule */
  get schedule() { return this.q.schedule({}) }
  /** take over a SNIP20 token */
  acquire = async (snip20: any) => {
    const tx1 = await snip20.setMinters([this.address]);
    const tx2 = await snip20.changeAdmin(this.address);
    return [tx1, tx2] }
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

export class RPTContract extends ScrtContract {

  static schema = loadSchemas(import.meta.url, {
    initMsg:     "./rpt/init.json",
    queryMsg:    "./rpt/query.json",
    queryAnswer: "./rpt/response.json",
    handleMsg:   "./rpt/handle.json"
  })

  code = { ...this.code, workspace: abs(), crate: 'sienna-rpt' }

  init = { ...this.init, label: 'SiennaRPT', msg: {} }

  constructor (options: {
    prefix?:  string
    admin?:   Agent
    config?:  LinearMapFor_HumanAddrAnd_Uint128
    portion?: Uint128
    SIENNA?:  SiennaSNIP20
    MGMT?:    MGMTContract
  } = {}) {
    super({
      prefix: options?.prefix,
      agent:  options?.admin,
      schema: RPTContract.schema
    })
    Object.assign(this.init.msg, {
      token:   options?.SIENNA?.linkPair,
      mgmt:    options?.MGMT?.linkPair,
      portion: options.portion,
      config:  [[options.admin?.address, options.portion]]
    })
    Object.defineProperties(this.init.msg, {
      token: { enumerable: true, get () { return options?.SIENNA?.linkPair } },
      mgmt:  { enumerable: true, get () { return options?.MGMT?.linkPair   } }
    })
  }

  /** query contract status */
  get status() { return this.q.status().then(({status})=>status) }

  /** set the splitt proportions */
  configure = (config = []) => this.tx.configure({ config })

  /** claim portions from mgmt and distribute them to recipients */
  vest = () => this.tx.vest()

  /** set the admin */
  setOwner = (new_admin) => this.tx.set_owner({ new_admin })

  static attach = (
    address:  string,
    codeHash: string,
    agent:    Agent
  ) => {
    const instance = new RPTContract({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }

}

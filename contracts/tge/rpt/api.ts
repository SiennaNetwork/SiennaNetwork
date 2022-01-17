import type { IAgent } from '@fadroma/scrt'
import type { SNIP20Contract_1_0 } from '@fadroma/snip20'
import { ScrtContract_1_0, loadSchemas } from "@fadroma/scrt"

import type { MGMTContract } from '@sienna/mgmt'
import { workspace } from '@sienna/settings'

import type { LinearMapFor_HumanAddrAnd_Uint128, Uint128 } from './rpt/init'

export type RPTOptions = {
  prefix?:  string
  admin?:   IAgent
  config?:  LinearMapFor_HumanAddrAnd_Uint128
  portion?: Uint128
  SIENNA?:  SNIP20Contract_1_0
  MGMT?:    MGMTContract
}

export class RPTContract extends ScrtContract_1_0 {

  static schema = loadSchemas(import.meta.url, {
    initMsg:     "./schema/init.json",
    queryMsg:    "./schema/query.json",
    queryAnswer: "./schema/response.json",
    handleMsg:   "./schema/handle.json"
  })

  code = { ...this.code, workspace, crate: 'sienna-rpt' }

  init = { ...this.init, label: 'SiennaRPT', msg: {} }

  constructor (options: RPTOptions = {}) {

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
    agent:    IAgent
  ) => {
    const instance = new RPTContract({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }

}


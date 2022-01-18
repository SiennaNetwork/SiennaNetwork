import type { IAgent, ContractState } from '@fadroma/scrt'
import type { SNIP20Contract_1_0 } from '@fadroma/snip20'
import { ScrtContract_1_0 } from "@fadroma/scrt"

import type { MGMTContract } from '@sienna/mgmt'
import { workspace } from '@sienna/settings'

import type { LinearMapFor_HumanAddrAnd_Uint128, Uint128 } from './rpt/init'

export class RPTContract extends ScrtContract_1_0 {

  crate = 'sienna-rpt'

  name = 'SiennaRPT'

  constructor (options: ContractState & {
    admin?:   IAgent,
    config?:  LinearMapFor_HumanAddrAnd_Uint128
    portion?: Uint128
    SIENNA?:  SNIP20Contract_1_0
    MGMT?:    MGMTContract
  } = {}) {

    super(options)

    Object.assign(this.initMsg, {
      token:   options?.SIENNA?.linkPair,
      mgmt:    options?.MGMT?.linkPair,
      portion: options.portion,
      config:  [[options.admin?.address, options.portion]]
    })

    Object.defineProperties(this.initMsg, {
      token: { enumerable: true, get () { return options?.SIENNA?.linkPair } },
      mgmt:  { enumerable: true, get () { return options?.MGMT?.linkPair   } }
    })

  }

  /** query contract status */
  get status() {
    return this.q.status().then(({status})=>status)
  }

  /** set the splitt proportions */
  configure (config = []) {
    return this.tx.configure({ config })
  }

  /** claim portions from mgmt and distribute them to recipients */
  vest () {
    return this.tx.vest()
  }

  /** set the admin */
  setOwner (new_admin) {
    return this.tx.set_owner({ new_admin })
  }

}

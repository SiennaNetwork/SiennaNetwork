import type { IAgent } from '@fadroma/scrt'
import { workspace } from '@sienna/settings'
import { SNIP20Contract_1_0 } from '@fadroma/snip20'
import { randomHex } from '@hackbg/tools'

export class SiennaSNIP20Contract extends SNIP20Contract_1_0 {

  code = { ...this.code, workspace, crate: "snip20-sienna" }

  init = {
    ...this.init,
    label: this.init.label || `SiennaSNIP20`,
    msg: {
      name: "Sienna",
      symbol: "SIENNA",
      decimals: 18,
      config: { public_total_supply: true }
    }
  }

  constructor (options: {
    prefix?: string,
    admin?:  IAgent
  } = {}) {
    super({
      prefix: options?.prefix,
      agent:  options?.admin
    })
    this.init.msg.prng_seed = randomHex(36)
  }

}

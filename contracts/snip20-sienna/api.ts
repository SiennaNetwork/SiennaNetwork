import type { IAgent } from '@fadroma/ops'
import { workspace } from '@sienna/settings'
import { SNIP20Contract } from '@fadroma/snip20'
import { randomHex } from '@fadroma/tools'

export class SiennaSNIP20Contract extends SNIP20Contract {

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

  static attach = (
    address:  string,
    codeHash: string,
    agent:    IAgent
  ) => {
    const instance = new SiennaSNIP20Contract({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }

}
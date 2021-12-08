import type { Agent } from '@fadroma/ops'
import { SNIP20 } from './SNIP20'
import { abs } from '../ops/index'
import { randomHex } from '@fadroma/tools'

export class SiennaSNIP20 extends SNIP20 {

  code = {
    ...this.code,
    workspace: abs(),
    crate: "snip20-sienna"
  }

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

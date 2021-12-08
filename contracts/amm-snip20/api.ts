import type { Agent } from "@fadroma/scrt";
import { randomHex } from "@fadroma/tools";
import { abs } from "../ops/index";

export class AMMSNIP20 extends SNIP20 {

  code = {
    ...this.code,
    workspace: abs(),
    crate: "amm-snip20"
  };

  init = {
    ...this.init,
    label: this.init.label || `AMMSNIP20`,
    msg: {
      get prng_seed() {
        return randomHex(36);
      },
      name: "Sienna",
      symbol: "SIENNA",
      decimals: 18,
      config: { 
        public_total_supply: true,
        enable_mint: true
      },
    },
  };

  constructor (options: {
    prefix?: string,
    admin?:  Agent
  } = {}) {
    super({
      prefix: options?.prefix,
      agent:  options?.admin
    })
  }

  static attach = (
    address:  string,
    codeHash: string,
    agent:    Agent
  ) => {
    const instance = new AMMSNIP20({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }
}

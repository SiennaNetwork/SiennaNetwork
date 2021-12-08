import type { IAgent } from "@fadroma/scrt";
import { randomHex } from "@fadroma/tools";
import { SNIP20Contract } from "@fadroma/snip20";
import { workspace } from "@sienna/settings";

export class AMMSNIP20Contract extends SNIP20Contract {

  code = { ...this.code, workspace, crate: "amm-snip20" };

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
    admin?:  IAgent
  } = {}) {
    super({
      prefix: options?.prefix,
      agent:  options?.admin
    })
  }

  static attach = (
    address:  string,
    codeHash: string,
    agent:    IAgent
  ) => {
    const instance = new AMMSNIP20({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }
}

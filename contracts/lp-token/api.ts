import type { Agent } from "@fadroma/scrt";
import { randomHex } from "@fadroma/tools";
import { abs } from "../ops/index";

const lpTokenDefaultConfig = {
  enable_deposit: true,
  enable_redeem: true,
  enable_mint: true,
  enable_burn: true,
  public_total_supply: true,
};

export class LPToken extends SNIP20 {
  code = {
    ...this.code,
    workspace: abs(),
    crate: "lp-token"
  }

  init = {
    ...this.init,
    label: this.init.label || `LP`,
    msg: {
      name:     "Liquidity Provision Token",
      symbol:   "LPTOKEN",
      decimals: 18,
      config:   { ...lpTokenDefaultConfig },
      get prng_seed() { return randomHex(36); },
    },
  }

  constructor(options: {
    admin:   Agent,
    name?:   string,
    prefix?: string,
  } = {}) {
    super({
      agent:  options?.admin,
      prefix: options?.prefix,
      label: `SiennaRewards_${options?.name}_LPToken`,
      initMsg: {
        symbol: `LP-${options?.name}`,
        name: `${options?.name} liquidity provision token`,
      },
    })
  }

  static attach = (
    address:  string,
    codeHash: string,
    agent:    Agent
  ) => {
    const instance = new LPToken({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }
}

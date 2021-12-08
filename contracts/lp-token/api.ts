import type { Agent } from "@fadroma/scrt";
import { randomHex } from "@fadroma/tools";
import { SNIP20Contract } from "@fadroma/snip20";
import { workspace } from "@sienna/settings";

export const defaultConfig = {
  enable_deposit:      true,
  enable_redeem:       true,
  enable_mint:         true,
  enable_burn:         true,
  public_total_supply: true,
};

export type LPTokenOptions = {
  admin?:  Agent,
  name?:   string,
  prefix?: string,
}

export class LPTokenContract extends SNIP20Contract {
  code = { ...this.code, workspace, crate: "lp-token" }

  init = {
    ...this.init,
    label: this.init.label || `LP`,
    msg: {
      name:     "Liquidity Provision Token",
      symbol:   "LPTOKEN",
      decimals: 18,
      config:   { ...defaultConfig },
      get prng_seed() { return randomHex(36); },
    },
  }

  constructor(options: LPTokenOptions = {}) {
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
    const instance = new LPTokenContract({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }
}

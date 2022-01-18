import type { IAgent, ContractState } from "@fadroma/scrt"
import { randomHex } from "@hackbg/tools"
import { SNIP20Contract_1_2 } from "@fadroma/snip20"
import { workspace } from "@sienna/settings"
import { InitMsg } from "./schema/init_msg.d"

export type LPTokenOptions = {
  admin?:  IAgent,
  name?:   string,
  prefix?: string,
}

export class LPTokenContract extends SNIP20Contract_1_2 {

  crate = 'lp-token'

  name = 'LP'

  initMsg: InitMsg = {
    name:     "Liquidity Provision Token",
    symbol:   "LPTOKEN",
    decimals:  18,
    config:    {
      enable_deposit:      true,
      enable_redeem:       true,
      enable_mint:         true,
      enable_burn:         true,
      public_total_supply: true,
    },
    prng_seed: randomHex(36),
  }

  constructor (options) {
    super(options)
    if (options.name) {
      this.name           = `SiennaRewards_${options?.name}_LPToken`
      this.initMsg.name   = `${options?.name} liquidity provision token`
      this.initMsg.symbol = `LP-${options?.name}`
    }
  }
}

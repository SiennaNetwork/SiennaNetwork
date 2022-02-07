import { Agent, randomHex, Snip20Contract_1_2 } from "@hackbg/fadroma"
import { workspace } from '@sienna/settings'
import { InitMsg } from "./schema/init_msg.d"

export type LPTokenOptions = {
  admin?:  Agent,
  name?:   string,
  prefix?: string,
}

import { LPTokenClient } from './LPTokenClient'
export { LPTokenClient }
export class LPTokenContract extends Snip20Contract_1_2 {
  source = { workspace, crate: 'lp-token' }
  Client = LPTokenClient
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
  constructor (options = {}) {
    super(options)
    if (options.name) {
      this.name           = `SiennaRewards_${options?.name}_LPToken`
      this.initMsg.name   = `${options?.name} liquidity provision token`
      this.initMsg.symbol = `LP-${options?.name}`
    }
  }
}

import { Console, bold, timestamp, randomHex } from "@hackbg/fadroma"

const console = Console('@sienna/lp-token')

import { Agent, Snip20Contract_1_2 } from "@hackbg/fadroma"
import { workspace } from '@sienna/settings'
import { InitMsg } from "./schema/init_msg.d"
import type { AMMVersion } from '@sienna/exchange'

export type LPTokenOptions = {
  admin?:  Agent,
  name?:   string,
  prefix?: string,
}

import { LPTokenClient } from './LPTokenClient'
export { LPTokenClient }
export class LPTokenContract extends Snip20Contract_1_2 {

  static "v1" = class AMMExchangeContract_v1 extends LPTokenContract {
    name   = 'AMM[v1].LPToken'
    version = "v1" as AMMVersion
    source  = { workspace, crate: 'lp-token', ref: 'a99d8273b4' }
  }

  static "v2" = class AMMExchangeContract_v2 extends LPTokenContract {
    name   = 'AMM[v2].LPToken'
    version = "v2" as AMMVersion
    source  = { workspace, crate: 'lp-token', ref: '39e87e4' }
  }

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

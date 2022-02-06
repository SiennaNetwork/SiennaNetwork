import { Agent, randomHex, Snip20Contract_1_2 } from "@hackbg/fadroma"
import { workspace } from '@sienna/settings'
import { InitMsg } from "./schema/init_msg.d"

export type LPTokenOptions = {
  admin?:  Agent,
  name?:   string,
  prefix?: string,
}

export class LPTokenContract extends Snip20Contract_1_2 {

  source = { workspace, crate: 'lp-token' }

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

  get friendlyName (): Promise<string> {
    const { chain, agent } = this
    return this.info.then(async ({name})=>{
      const fragments = name.split(' ')
      const [t0addr, t1addr] = fragments[fragments.length-1].split('-')
      const t0 = new Snip20Contract_1_2({ chain, agent, address: t0addr })
      const t1 = new Snip20Contract_1_2({ chain, agent, address: t1addr })
      const [t0info, t1info] = await Promise.all([t0.info, t1.info])
      return `LP-${t0info.symbol}-${t1info.symbol}`
    })
  }

}

import type { IAgent, ContractState } from "@fadroma/scrt"
import { randomHex } from "@fadroma/scrt"
import { SNIP20Contract_1_2 } from "@fadroma/snip20"
import { InitMsg } from './schema/init_msg.d'

export class AMMSNIP20Contract extends SNIP20Contract_1_2 {

  crate = 'amm-snip20'

  name = 'AMMSNIP20'

  initMsg: InitMsg = {
    prng_seed: randomHex(36),
    name:      "AMSNIP20",
    symbol:    "AMM",
    decimals:  18,
    config:    {
      public_total_supply: true,
      enable_mint: true
    },
  }

  constructor (options: ContractState & {
    admin?:  IAgent
  } = {}) {
    super(options)
  }

}

import { Agent, ContractState, randomHex } from "@hackbg/fadroma"
import { SNIP20Contract_1_2 } from "@fadroma/snip20"
import { InitMsg } from './schema/init_msg.d'

export class AMMSNIP20Contract extends SNIP20Contract_1_2 {

  crate = 'amm-snip20'

  name  = 'AMMSNIP20'

  initMsg: InitMsg = {
    prng_seed: randomHex(36),
    name:      "AMMSNIP20",
    symbol:    "AMM",
    decimals:  18,
    config:    {
      public_total_supply: true,
      enable_mint: true
    },
  }

}

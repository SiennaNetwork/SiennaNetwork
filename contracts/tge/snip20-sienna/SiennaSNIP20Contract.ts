import type { IAgent } from '@fadroma/scrt'
import { workspace } from '@sienna/settings'
import { SNIP20Contract_1_0, SNIP20Executor } from '@fadroma/snip20'
import { randomHex } from '@hackbg/tools'
import { InitMsg } from './schema/init_msg.d'

export class SiennaSNIP20Contract extends SNIP20Contract_1_0 {

  crate = 'snip20-sienna'

  name  = 'SiennaSNIP20'

  initMsg: InitMsg = {
    name:      "Sienna",
    symbol:    "SIENNA",
    decimals:  18,
    config:    { public_total_supply: true },
    prng_seed: randomHex(36)
  }

}

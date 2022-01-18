import { ScrtContract_1_2 } from "@fadroma/scrt"

import { InitMsg } from './schema/init_msg.d'

export class LendMarketContract extends ScrtContract_1_2 {

  crate = 'lend-market'

  name  = 'SiennaLendMarket'

  initMsg?: InitMsg

}

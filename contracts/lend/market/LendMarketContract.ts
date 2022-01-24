import { Scrt_1_2 } from "@hackbg/fadroma"

import { InitMsg } from './schema/init_msg.d'

export class LendMarketContract extends Scrt_1_2.Contract<any, any> {

  crate = 'lend-market'

  name  = 'SiennaLendMarket'

  initMsg?: InitMsg

}

import { Scrt_1_2 } from "@hackbg/fadroma"

import { InitMsg } from './schema/init_msg.d'

export class LendOracleContract extends Scrt_1_2.Contract<any, any> {

  crate = 'lend-oracle'

  name  = 'SiennaLendOracle'

  initMsg?: InitMsg

}
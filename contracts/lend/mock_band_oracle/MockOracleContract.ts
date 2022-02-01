import { Scrt_1_2 } from "@hackbg/fadroma"

import { InitMsg } from './schema/init_msg'

export class MockOracleContract extends Scrt_1_2.Contract<any, any> {

  crate = 'lend-mock-oracle'

  name  = 'SiennaLendMockOracle'

  initMsg?: InitMsg

}

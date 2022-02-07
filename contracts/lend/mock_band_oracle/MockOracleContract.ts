import { Scrt_1_2 } from "@hackbg/fadroma"
import { workspace } from "@sienna/settings"
import { InitMsg } from './schema/init_msg'

export class MockOracleContract extends Scrt_1_2.Contract<any> {
  name   = 'SiennaLendMockOracle'
  source = { workspace, crate: 'lend-mock-oracle' }
  initMsg?: InitMsg
}

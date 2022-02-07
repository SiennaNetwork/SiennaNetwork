import { Scrt_1_2 } from "@hackbg/fadroma"
import { workspace } from "@sienna/settings"
import { InitMsg } from './schema/init_msg.d'

export class LendOracleContract extends Scrt_1_2.Contract<any> {
  name   = 'SiennaLendOracle'
  source = { workspace, crate: 'lend-oracle' }
  initMsg?: InitMsg
}

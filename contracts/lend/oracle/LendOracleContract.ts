import { ScrtContract_1_2 } from "@fadroma/scrt"

import { workspace } from '@sienna/settings'

import { InitMsg } from './schema/init_msg.d'

export class LendOracleContract extends ScrtContract_1_2 {

  crate = 'lend-oracle'

  name  = 'SiennaLendOracle'

  initMsg?: InitMsg

}

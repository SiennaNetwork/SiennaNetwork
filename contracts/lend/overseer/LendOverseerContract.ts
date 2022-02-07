import { Scrt_1_2 } from "@hackbg/fadroma"
import { workspace } from "@sienna/settings"
import { InitMsg } from './schema/init_msg.d'

export class LendOverseerContract extends Scrt_1_2.Contract<any> {
  name   = 'SiennaLendOverseer'
  source = { workspace, crate: 'lend-overseer' }
  initMsg?: InitMsg
}

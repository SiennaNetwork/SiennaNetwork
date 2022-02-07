import { Scrt_1_2 } from "@hackbg/fadroma"
import { workspace } from "@sienna/settings"
import { InitMsg } from './schema/init_msg.d'

export class InterestModelContract extends Scrt_1_2.Contract<any> {
  name   = 'SiennaLendInterestModel'
  source = { workspace, crate: 'lend-interest-model' }
  initMsg?: InitMsg
}

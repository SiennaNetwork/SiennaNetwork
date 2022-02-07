import { Scrt_1_2 } from "@hackbg/fadroma"
import { workspace } from '@sienna/settings'

export class SwapRouterContract extends Scrt_1_2.Contract<any> {

  name = 'AMM.Router'

  source = { workspace, crate: 'router' }

}

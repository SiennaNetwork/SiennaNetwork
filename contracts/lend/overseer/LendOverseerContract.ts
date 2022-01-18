import { randomHex, ScrtContract_1_2, IAgent, ContractState } from "@fadroma/scrt"

import { workspace } from '@sienna/settings'

import { InitMsg } from './schema/init_msg.d'

export class LendOverseerContract extends ScrtContract_1_2 {

  crate = 'lend-overseer'

  name  = 'SiennaLendOverseer'

  initMsg?: InitMsg

}

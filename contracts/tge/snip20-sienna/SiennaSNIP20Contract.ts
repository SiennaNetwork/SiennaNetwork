import { SNIP20Contract_1_0 } from '@hackbg/fadroma'
import { workspace } from '@sienna/settings'
import { InitMsg } from './schema/init_msg.d'

export class SiennaSNIP20Contract extends SNIP20Contract_1_0 {
  workspace = workspace
  crate = 'snip20-sienna'
  name  = 'SIENNA'
  initMsg: InitMsg
}

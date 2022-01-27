import { Agent, randomHex } from '@hackbg/fadroma'
import { SNIP20Contract_1_0 } from '@fadroma/snip20'
import { workspace } from '@sienna/settings'
import { InitMsg } from './schema/init_msg.d'

export class SiennaSNIP20Contract extends SNIP20Contract_1_0 {
  crate = 'snip20-sienna'
  name  = 'SiennaSNIP20'
  initMsg: InitMsg
}

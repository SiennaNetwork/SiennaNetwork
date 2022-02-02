import { SNIP20Contract_1_0, Console, bold } from '@hackbg/fadroma'
import { workspace } from '@sienna/settings'
import { InitMsg } from './schema/init_msg.d'

const console = Console('@sienna/snip20-sienna')

export class SiennaSNIP20Contract extends SNIP20Contract_1_0 {
  workspace = workspace
  crate = 'snip20-sienna'
  name  = 'SIENNA'
  initMsg: InitMsg

  static status = async function siennaStatus ({
    deployment, agent,
    SIENNA = deployment.getThe('SIENNA', new SiennaSNIP20Contract({ agent }))
  }) {
    try {
      console.info(`SIENNA balance of ${bold(agent.address)}: ${await SIENNA.q().balance(agent.address, '')}`)
    } catch (e) {
      console.error(e.message)
    }
  }
}

import type { Agent } from '@fadroma/ops'
import type { MGMT } from './MGMT'
import type { SiennaSNIP20 } from "./SNIP20"
import type { LinearMapFor_HumanAddrAnd_Uint128, Uint128 } from './rpt/init'
import { ScrtContract, loadSchemas } from "@fadroma/scrt"
import { abs } from '../ops/index'

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./rpt/init.json",
  queryMsg: "./rpt/query.json",
  queryAnswer: "./rpt/response.json",
  handleMsg: "./rpt/handle.json",
});

export class RPT extends ScrtContract {

  constructor (options: {
    admin:   Agent,
    config:  LinearMapFor_HumanAddrAnd_Uint128,
    portion: Uint128,
    SIENNA:  SiennaSNIP20
    MGMT:    MGMT
  }) {
    super({ agent: options.admin, schema })
    Object.assign(this.init.msg, {
      token: options.SIENNA.linkPair,
      mgmt: options.MGMT.linkPair,
      portion: options.portion,
      config: [[options.admin.address, options.portion]]
    })
  }

  code = { ...this.code, workspace: abs(), crate: 'sienna-rpt' }

  init = { ...this.init, label: 'SiennaRPT', msg: {} }

  /** query contract status */
  get status() {
    return this.q.status();
  }

  /** set the splitt proportions */
  configure = (config = []) => this.tx.configure({ config });

  /** claim portions from mgmt and distribute them to recipients */
  vest = () => this.tx.vest();

  /** set the admin */
  setOwner = (new_admin) => this.tx.set_owner({ new_admin });
}

import type { ContractAPIOptions } from "@fadroma/scrt"
import { ScrtContract, loadSchemas } from "@fadroma/scrt"
import { abs } from '../ops/index'

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./rpt/init.json",
  queryMsg: "./rpt/query.json",
  queryAnswer: "./rpt/response.json",
  handleMsg: "./rpt/handle.json",
});

export class RPT extends ScrtContract {
  constructor(options: ContractAPIOptions = {}) { super({ ...options, schema }) }

  code = { ...super.code, workspace: abs(), crate: 'sienna-rpt' }
  init = { ...super.init, label: 'SiennaRPT', msg: {} }

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

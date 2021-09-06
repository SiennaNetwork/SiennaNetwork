import type { Agent } from "@fadroma/scrt"
import { ContractAPI, loadSchemas } from "@fadroma/scrt"

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./rpt/init.json",
  queryMsg: "./rpt/query.json",
  queryAnswer: "./rpt/response.json",
  handleMsg: "./rpt/handle.json",
});

export default class RPT extends ContractAPI {
  constructor(agent: Agent) {
    super(schema, agent);
  }

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

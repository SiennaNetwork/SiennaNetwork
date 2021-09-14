import { ScrtContract, loadSchemas, Agent } from "@fadroma/scrt"

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./ido/init_msg.json",
  queryMsg: "./ido/query_msg.json",
  queryAnswer: "./ido/query_response.json",
  handleMsg: "./ido/handle_msg.json",
});

export default class IDO extends ScrtContract {
  constructor (agent: Agent) { super(schema, agent) }

  swap(amount: string|number|bigint, agent: Agent) {
    return this.tx.swap(
      { amount: `${amount}` },
      agent,
      undefined,
      JSON.stringify([{ amount: `${amount}`, denom: "uscrt" }])
    );
  }
}

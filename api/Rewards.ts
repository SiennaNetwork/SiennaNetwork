import { ScrtContract, loadSchemas, Agent } from "@fadroma/scrt"

export const schema = loadSchemas(import.meta.url, {
  initMsg:     "./rewards/init.json",
  queryMsg:    "./rewards/query.json",
  queryAnswer: "./rewards/response.json",
  handleMsg:   "./rewards/handle.json",
});

export default class Rewards extends ScrtContract {
  constructor (agent: Agent) { super(schema, agent) }

  setProvidedToken = (address: string, code_hash: string, agent = this.instantiator) =>
    this.tx.set_provided_token({address, code_hash}, agent);

  lock = (amount: string, agent: Agent) =>
    this.tx.lock({ amount: String(amount) }, agent);

  retrieve = (amount: string, agent: Agent) =>
    this.tx.retrieve({ amount: String(amount) }, agent);

  claim = (agent: string) =>
    this.tx.claim({}, agent);
}

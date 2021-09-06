import { ScrtContract, loadSchemas } from "@fadroma/scrt"

export const schema = loadSchemas(import.meta.url, {
  initMsg:     "./rewards/init.json",
  queryMsg:    "./rewards/query.json",
  queryAnswer: "./rewards/response.json",
  handleMsg:   "./rewards/handle.json",
});

export default class Rewards extends ScrtContract {
  constructor(options = {}) { super(options, schema) }

  setProvidedToken = (address, code_hash, agent = this.agent) =>
    this.tx.set_provided_token({address, code_hash}, agent);

  lock = (amount, agent) =>
    this.tx.lock({ amount: String(amount) }, agent);

  retrieve = (amount, agent) =>
    this.tx.retrieve({ amount: String(amount) }, agent);

  claim = (agent) =>
    this.tx.claim({}, agent);
}

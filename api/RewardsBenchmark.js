import { SecretNetworkContractWithSchema } from "@fadroma/scrt-agent";
import { loadSchemas } from "@fadroma/utilities";

export const schema = loadSchemas(import.meta.url, {
  initMsg:     "./rewards-benchmark/init.json",
  queryMsg:    "./rewards-benchmark/query.json",
  queryAnswer: "./rewards-benchmark/response.json",
  handleMsg:   "./rewards-benchmark/handle.json",
});
const decoder = new TextDecoder();
const decode  = (buffer) => decoder.decode(buffer).trim();

export default class RewardsBenchmarkContract extends SecretNetworkContractWithSchema {
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

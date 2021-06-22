import { SecretNetworkContractWithSchema } from "@fadroma/scrt-agent";
import { loadSchemas } from "@fadroma/utilities";

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./rewards/init_msg.json",
  queryMsg: "./rewards/query_msg.json",
  queryAnswer: "./rewards/query_msg_response.json",
  handleMsg: "./rewards/handle_msg.json",
});
const decoder = new TextDecoder();
const decode = (buffer) => decoder.decode(buffer).trim();

export default class RewardsContract extends SecretNetworkContractWithSchema {
  constructor(options = {}) {
    super(options, schema);
  }

  get status() {
    return this.q.status();
  }

  get admin() {
    return this.q.admin("admin");
  }

  getTotalRewardsSupply = () => this.q.total_rewards_supply();

  getAccounts = (address, lp_tokens, viewing_key, agent) =>
    this.q.accounts({ address, lp_tokens, viewing_key }, agent);

  simulate = (address, current_time, lp_tokens, viewing_key, agent) =>
    this.q.claim_simulation(
      { address, current_time, lp_tokens, viewing_key },
      agent
    );

  changeAdmin = (address, agent) =>
    this.tx.admin({ change_admin: address }, agent);

  lock = (amount, lp_token, agent) =>
    this.tx.lock_tokens({ amount: String(amount), lp_token }, agent);

  retrieve = (amount, lp_token, agent) =>
    this.tx.retrieve_tokens({ amount: String(amount), lp_token }, agent);

  claim = (lp_tokens, agent) => this.tx.claim({ lp_tokens }, agent);

  changePools = (pools, total_share, agent) =>
    this.tx.change_pools({ pools, total_share }, agent);

  createViewingKey = (
    agent,
    entropy = "" //randomHex(32)) =>
  ) =>
    this.tx.create_viewing_key({ entropy }, agent).then((tx) => ({
      tx,
      key: JSON.parse(decode(tx.data)).create_viewing_key.key,
    }));

  setViewingKey = (agent, key) => this.tx.set_viewing_key({ key }, agent);
}

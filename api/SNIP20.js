import { randomBytes } from "crypto";
import { SecretNetworkContractWithSchema } from "@fadroma/scrt-agent";
import { loadSchemas } from "@fadroma/utilities";

const randomHex = (bytes) => randomBytes(bytes).toString("hex");

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./snip20/init_msg.json",
  queryMsg: "./snip20/query_msg.json",
  queryAnswer: "./snip20/query_answer.json",
  handleMsg: "./snip20/handle_msg.json",
  handleAnswer: "./snip20/handle_answer.json",
});

const decoder = new TextDecoder();
const decode = (buffer) => decoder.decode(buffer).trim();

export default class SNIP20 extends SecretNetworkContractWithSchema {
  constructor(options = {}) {
    super(options, schema);
  }

  changeAdmin = (address, agent) => this.tx.change_admin({ address }, agent);

  setMinters = (minters, agent) => this.tx.set_minters({ minters }, agent);

  addMinters = (minters, agent) =>
    this.tx.add_minters({ minters, padding: null }, agent);

  mint = (amount, agent = this.agent, recipient = agent.address) =>
    this.tx
      .mint({ amount: String(amount), recipient, padding: null }, agent)
      .then((tx) => ({ tx, mint: JSON.parse(decode(tx.data)).mint }));

  balance = async (address, key) => {
    const response = await this.q.balance({ address, key });

    if (response.balance && response.balance.amount) {
      return response.balance.amount;
    } else {
      throw new Error(JSON.stringify(response));
    }
  };

  createViewingKey = (
    agent,
    entropy = "" //randomHex(32)) =>
  ) =>
    this.tx
      .create_viewing_key({ entropy, padding: null }, agent)
      .then((tx) => ({
        tx,
        key: JSON.parse(decode(tx.data)).create_viewing_key.key,
      }));

  setViewingKey = (agent, key) =>
    this.tx
      .set_viewing_key({ key }, agent)
      .then((tx) => ({
        tx,
        status: JSON.parse(decode(tx.data)).set_viewing_key.key,
      }));

  increaseAllowance = (amount, spender, agent) =>
    this.tx.increase_allowance({ amount: String(amount), spender }, agent);

  decreaseAllowance = (amount, spender, agent) =>
    this.tx.decrease_allowance({ amount: String(amount), spender }, agent);

  checkAllowance = (spender, owner, key, agent) =>
    this.q.allowance({ owner, spender, key }, agent);
}

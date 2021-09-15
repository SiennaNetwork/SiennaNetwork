import type { Agent } from "@fadroma/scrt"
import { ScrtContract, loadSchemas } from "@fadroma/scrt"
import { randomHex } from "@fadroma/tools"
import { abs } from '../ops/index'

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./snip20/init_msg.json",
  queryMsg: "./snip20/query_msg.json",
  queryAnswer: "./snip20/query_answer.json",
  handleMsg: "./snip20/handle_msg.json",
  handleAnswer: "./snip20/handle_answer.json",
});

const decoder = new TextDecoder();
const decode = (buffer: any) => decoder.decode(buffer).trim();

export class SNIP20 extends ScrtContract {

  /* Get an API wrapper for an existing contract */
  static attach (agent: Agent, address: string, codeHash: string) {
    const instance = new this(agent)
    instance.code.codeHash = codeHash
    instance.init.address  = address
    return instance
  }

  constructor(agent: Agent) {
    super(schema, agent);
  }

  changeAdmin = (address: string, agent?: Agent) =>
    this.tx.change_admin({ address }, agent);

  setMinters = (minters: Array<string>, agent?: Agent) =>
    this.tx.set_minters({ minters }, agent);

  addMinters = (minters: Array<string>, agent?: Agent) =>
    this.tx.add_minters({ minters, padding: null }, agent);

  mint = (amount: string, agent = this.instantiator, recipient = agent.address) =>
    this.tx
      .mint({ amount: String(amount), recipient, padding: null }, agent)
      .then((tx) => ({ tx, mint: JSON.parse(decode(tx.data)).mint }));

  balance = async (address: string, key: string) => {
    const response = await this.q.balance({ address, key });

    if (response.balance && response.balance.amount) {
      return response.balance.amount;
    } else {
      throw new Error(JSON.stringify(response));
    }
  };

  createViewingKey = (
    agent: Agent,
    entropy = randomHex(32)
  ) =>
    this.tx
      .create_viewing_key({ entropy, padding: null }, agent)
      .then((tx) => ({
        tx,
        key: JSON.parse(decode(tx.data)).create_viewing_key.key,
      }));

  setViewingKey = (agent: Agent, key: string) =>
    this.tx
      .set_viewing_key({ key }, agent)
      .then((tx) => ({
        tx,
        status: JSON.parse(decode(tx.data)).set_viewing_key.key,
      }));

  increaseAllowance = (amount: string, spender: string, agent: Agent) =>
    this.tx.increase_allowance({ amount: String(amount), spender }, agent);

  decreaseAllowance = (amount: string, spender: string, agent: Agent) =>
    this.tx.decrease_allowance({ amount: String(amount), spender }, agent);

  checkAllowance = (spender: string, owner: string, key: string, agent: Agent) =>
    this.q.allowance({ owner, spender, key }, agent);
  
  /**
   * Perform send with a callback message that will be sent to IDO contract
   * 
   * @param {string} contractAddress Address of the IDO contract where we will send this amount
   * @param {string} amount Amount to send
   * @param {string} [recipient] Recipient of the bought funds from IDO contract
   * @param {Agent} [agent] 
   * @returns 
   */
  sendIdo = (contractAddress, amount, recipient = null, agent) =>
    this.tx.send(
      {
        recipient: contractAddress,
        amount: `${amount}`,
        msg: Buffer.from(
          JSON.stringify({ swap: { recipient } }),
          "utf8"
        ).toString("base64"),
      },
      agent
    )
}

export class SiennaSNIP20 extends SNIP20 {
  code = { ...super.code, workspace: abs(), crate: 'snip20-sienna' }
  init = { ...super.init, label: 'SiennaSNIP20', msg: {
    get prng_seed () { return randomHex(36) },
    name:     "Sienna",
    symbol:   "SIENNA",
    decimals: 18,
    config:   { public_total_supply: true } } } }

export class AMMSNIP20 extends SNIP20 {
  code = { ...super.code, workspace: abs(), crate: 'amm-snip20' }
  init = { ...super.init, label: 'ExchangedSnip20', msg: {} } }

const lpTokenDefaultConfig = {
  enable_deposit: true, enable_redeem: true,
  enable_mint: true, enable_burn: true,
  public_total_supply: true }

export class LPToken extends SNIP20 {
  code = { ...super.code, workspace: abs(), crate: 'lp-token' }
  init = { ...super.init, label: `LP`, msg: {
    get prng_seed () { return randomHex(36) },
    name:     "Liquidity Provision Token",
    symbol:   "LP",
    decimals: 18,
    config:   { ...lpTokenDefaultConfig } } }
  constructor (agent: Agent, name: string) {
    super(agent)
    this.init.label      = `SiennaRewards_${name}_LPToken`
    this.init.msg.symbol = `LP-${name}`
    this.init.msg.name   = `${name} liquidity provision token` }}

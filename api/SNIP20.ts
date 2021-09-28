import type { Agent, ContractAPIOptions } from "@fadroma/scrt";
import { ScrtContract, loadSchemas } from "@fadroma/scrt";
import { randomHex } from "@fadroma/tools";
import { abs } from "../ops/index";

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./snip20/init_msg.json",
  queryMsg: "./snip20/query_msg.json",
  queryAnswer: "./snip20/query_answer.json",
  handleMsg: "./snip20/handle_msg.json",
  handleAnswer: "./snip20/handle_answer.json",
});

// @ts-ignore
const decoder = new TextDecoder();
const decode = (buffer: any) => decoder.decode(buffer).trim();

export class SNIP20 extends ScrtContract {
  constructor(options: ContractAPIOptions = {}) {
    super({ ...options, schema });
  }

  /**
   * Change admin of the token
   *
   * @param {string} address
   * @param {Agent} [agent]
   * @returns
   */
  changeAdmin = (address: string, agent?: Agent) =>
    this.tx.change_admin({ address }, agent);

  /**
   * Add addresses to be minters
   *
   * @param {string[]} minters
   * @param {Agent} [agent]
   * @returns
   */
  setMinters = (minters: Array<string>, agent?: Agent) =>
    this.tx.set_minters({ minters }, agent);

  /**
   * Set specific addresses to be minters, remove all others
   *
   * @param {string[]} minters
   * @param {Agent} [agent]
   * @returns
   */
  addMinters = (minters: Array<string>, agent?: Agent) =>
    this.tx.add_minters({ minters, padding: null }, agent);

  /**
   * Mint tokens
   * @param {string|number|bigint} amount
   * @param agent
   * @param recipient
   * @returns
   */
  mint = (
    amount: string | number | bigint,
    agent = this.instantiator,
    recipient = agent.address
  ) =>
    this.tx
      .mint({ amount: String(amount), recipient, padding: null }, agent)
      .then((tx) => ({ tx, mint: JSON.parse(decode(tx.data)).mint }));

  /**
   * Get address balance
   *
   * @param {string} address
   * @param {string} key
   * @returns
   */
  balance = async (address: string, key: string) => {
    const response = await this.q.balance({ address, key });

    if (response.balance && response.balance.amount) {
      return response.balance.amount;
    } else {
      throw new Error(JSON.stringify(response));
    }
  };

  /**
   * Create viewing key for the agent
   *
   * @param {Agent} agent
   * @param {string} entropy
   * @returns
   */
  createViewingKey = (agent: Agent, entropy = randomHex(32)) =>
    this.tx
      .create_viewing_key({ entropy, padding: null }, agent)
      .then((tx) => ({
        tx,
        key: JSON.parse(decode(tx.data)).create_viewing_key.key,
      }));

  /**
   * Set viewing key for the agent
   *
   * @param {Agent} agent
   * @param {string} key
   * @returns
   */
  setViewingKey = (agent: Agent, key: string) =>
    this.tx.set_viewing_key({ key }, agent).then((tx) => ({
      tx,
      status: JSON.parse(decode(tx.data)).set_viewing_key.key,
    }));

  /**
   * Increase allowance to spender
   * @param {string|number|bigint} amount
   * @param {string} spender
   * @param {Agent} [agent]
   * @returns
   */
  increaseAllowance = (
    amount: string | number | bigint,
    spender: string,
    agent?: Agent
  ) => this.tx.increase_allowance({ amount: String(amount), spender }, agent);

  /**
   * Decrease allowance to spender
   * @param {string|number|bigint} amount
   * @param {string} spender
   * @param {Agent} [agent]
   * @returns
   */
  decreaseAllowance = (
    amount: string | number | bigint,
    spender: string,
    agent?: Agent
  ) => this.tx.decrease_allowance({ amount: String(amount), spender }, agent);

  /**
   * Check available allowance
   *
   * @param {string} spender
   * @param {string} owner
   * @param {string} key
   * @param {Agent} [agent]
   * @returns
   */
  checkAllowance = (
    spender: string,
    owner: string,
    key: string,
    agent?: Agent
  ) => this.q.allowance({ owner, spender, key }, agent);

  /**
   * Perform send with a callback message that will be sent to IDO contract
   *
   * @param {string} contractAddress Address of the IDO contract where we will send this amount
   * @param {string|number|bigint} amount Amount to send
   * @param {string} [recipient] Recipient of the bought funds from IDO contract
   * @param {Agent} [agent]
   * @returns
   */
  sendIdo = (
    contractAddress: string,
    amount: string | number | bigint,
    recipient: string | null = null,
    agent?: Agent
  ) =>
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
    );

  /**
   * Perform locking of the funds in launchpad contract
   *
   * @param {string} contractAddress Address of the Launchpad contract where we will send this amount
   * @param {string|number|bigint} amount Amount to send
   * @param {Agent} [agent]
   * @returns
   */
  lockLaunchpad = (
    contractAddress: string,
    amount: string | number | bigint,
    agent?: Agent
  ) =>
    this.tx.send(
      {
        recipient: contractAddress,
        amount: `${amount}`,
        msg: Buffer.from(JSON.stringify({ lock: {} }), "utf8").toString(
          "base64"
        ),
      },
      agent
    );

  /**
   * Perform locking of the funds in launchpad contract
   *
   * @param {string} contractAddress Address of the Launchpad contract
   * @param {number} entries Number of entries to unlock
   * @param {Agent} [agent]
   * @returns
   */
  unlockLaunchpad = (contractAddress: string, entries: number, agent?: Agent) =>
    this.tx.send(
      {
        recipient: contractAddress,
        amount: `0`,
        msg: Buffer.from(
          JSON.stringify({ unlock: { entries } }),
          "utf8"
        ).toString("base64"),
      },
      agent
    );
}

export class SiennaSNIP20 extends SNIP20 {
  code = { ...this.code, workspace: abs(), crate: "snip20-sienna" };
  init = {
    ...this.init,
    label: this.init.label || "SiennaSNIP20",
    msg: {
      get prng_seed() {
        return randomHex(36);
      },
      name: "Sienna",
      symbol: "SIENNA",
      decimals: 18,
      config: { public_total_supply: true },
      
    },
  };
}

export class AMMSNIP20 extends SNIP20 {
  code = { ...this.code, workspace: abs(), crate: "amm-snip20" };
  init = { ...this.init, label: this.init.label || "AMMSNIP20" };
}

const lpTokenDefaultConfig = {
  enable_deposit: true,
  enable_redeem: true,
  enable_mint: true,
  enable_burn: true,
  public_total_supply: true,
};

export class LPToken extends SNIP20 {
  code = { ...this.code, workspace: abs(), crate: "lp-token" };
  init = {
    ...this.init,
    label: this.init.label || `LP`,
    msg: {
      get prng_seed() {
        return randomHex(36);
      },
      name: "Liquidity Provision Token",
      symbol: "LP",
      decimals: 18,
      config: { ...lpTokenDefaultConfig },
    },
  };
  constructor(options: ContractAPIOptions = {}, name: string = "???") {
    super({
      ...(options || {}),
      label: `SiennaRewards_${name}_LPToken`,
      initMsg: {
        ...(options?.initMsg || {}),
        symbol: `LP-${name}`,
        name: `${name} liquidity provision token`,
      },
    });
  }
}

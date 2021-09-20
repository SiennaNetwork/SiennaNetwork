import type { ContractAPIOptions } from '@fadroma/scrt'
import { ScrtContract, loadSchemas, Agent } from "@fadroma/scrt"
import { randomHex } from "@fadroma/tools"
import { abs } from '../ops/index'

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./launchpad/init_msg.json",
  queryMsg: "./launchpad/query_msg.json",
  queryAnswer: "./launchpad/query_response.json",
  handleMsg: "./launchpad/handle_msg.json",
});

// @ts-ignore
const decoder = new TextDecoder();
const decode = (buffer: any) => decoder.decode(buffer).trim();

export class Launchpad extends ScrtContract {
  constructor (options: ContractAPIOptions = {}) { super({ ...options, schema }) }

  code = { ...this.code, workspace: abs(), crate: 'launchpad' }

  /**
   * This method will perform the native token lock.
   *
   * @param {string|number|bigint} amount
   * @param {string} [denom]
   * @param {Agent} [agent]
   * @returns
   */
  async lock(amount: string|number|bigint, denom: string = "uscrt", agent?: Agent) {
    return this.tx.lock({ amount: `${amount}` }, agent, undefined, [
      { amount: `${amount}`, denom },
    ]);
  }

  /**
   * This method will perform the native token unlock
   *
   * @param {string|number|bigint} entries
   * @param {Agent} [agent]
   * @returns
   */
  async unlock(entries: string|number|bigint, agent?: Agent) {
    return this.tx.unlock({ entries }, agent);
  }

  /**
   * Get the configuration information about the Launchpad contract
   *
   * @returns Promise<{
   *  "token_type": { "native_token": { "denom": "uscrt" } },
   *  "segment": "25000000000",
   *  "bounding_period": 604800,
   *  "active": true,
   *  "token_decimals": 6,
   *  "locked_balance": "100000000000"
   * }[]>
   */
  async info() {
    return this.q.launchpad_info();
  }

  /**
   * Get the balance and entry information for a user
   *
   * @param {string} address
   * @param {string} key
   * @returns Promise<{
   *  "token_type": { "native_token": { "denom": "uscrt" } },
   *  "balance": "100000000000",
   *  "entries": [
   *    "1629402109",
   *    "1629402109",
   *    "1629402109",
   *    "1629402109",
   *  ],
   *  "last_draw": null,
   * }[]>
   */
  async userInfo(address: string, key: string) {
    return this.q.user_info({
      address,
      key,
    });
  }

  /**
   * Get the configuration information about the Launchpad contract
   *
   * @param {number} number
   * @param {string[]} tokens
   * @returns Promise<any>
   */
  async draw(number: number, tokens: string[]) {
    return this.q.draw({ number, tokens });
  }

  /**
   * Create viewing key for the agent
   * 
   * @param {Agent} agent 
   * @param {string} entropy 
   * @returns 
   */
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

  /**
   * Set viewing key for the agent
   * 
   * @param {Agent} agent 
   * @param {string} key 
   * @returns 
   */
  setViewingKey = (agent: Agent, key: string) =>
    this.tx
      .set_viewing_key({ key }, agent)
      .then((tx) => ({
        tx,
        status: JSON.parse(decode(tx.data)).set_viewing_key.key,
      }));
}

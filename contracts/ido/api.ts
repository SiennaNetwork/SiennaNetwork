import type { ContractAPIOptions } from '@fadroma/scrt'
import { ScrtContract, loadSchemas, Agent } from "@fadroma/scrt"
import { abs } from '../ops/index'

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./ido/init_msg.json",
  queryMsg: "./ido/query_msg.json",
  queryAnswer: "./ido/query_response.json",
  handleMsg: "./ido/handle_msg.json",
});

export type IDOOptions = ContractAPIOptions & { admin?: Agent }

export class IDO extends ScrtContract {
  constructor (options: IDOOptions = {}) {
    super({ ...options, agent: options.admin, schema })
  }

  code = { ...this.code, workspace: abs(), crate: 'ido' }

  /**
   * Check if the address can participate in an IDO
   * 
   * @param {string} address 
   * @returns Promise<{
   *  can_participate: bool
   * }>
   */
  eligibility(address: string) {
    return this.q.eligibility_info({ address });
  }

  /**
   * Check the sale status of the IDO project
   * 
   * @returns Promise<{
   *  total_allocation: string,
   *  available_for_sale: string,
   *  sold_in_pre_lock: string,
   *  is_active: bool,
   * }>
   */
  saleStatus() {
    return this.q.sale_status();
  }

  /**
   * Check the amount user has pre locked and the amount user has swapped
   * 
   * @param {string} address
   * @param {string} key
   * @returns Promise<{
   *  pre_lock_amount: string,
   *  total_bought: string,
   * }>
   */
  balance(address: string, key: string) {
    return this.q.balance({ address, key });
  }

  /**
   * Check the sale info of the IDO project
   * 
   * @returns Promise<{
   *  input_token: object, // same as init input token
   *  sold_token: object, // same as init sold token
   *  rate: string, // rate of exchange
   *  taken_seats: number,
   *  max_seats: number,
   *  max_allocation: string,
   *  min_allocation: string,
   *  start: number,
   *  end: number
   * }>
   */
  saleInfo() {
    return this.q.sale_info();
  }

  /**
   * This method will perform the native token swap.
   * 
   * IMPORTANT: if custom buy token is set, you have to use the SNIP20 
   * receiver callback interface to initiate swap.
   * 
   * @param {string|number|bigint} amount 
   * @param {Agent} [agent] 
   * @param {string|null} [receiver] 
   * @returns 
   */
  swap(amount: string|number|bigint, agent?: Agent, receiver: string|null = null) {
    return this.tx.swap(
      { amount: `${amount}`, receiver },
      agent,
      undefined,
      [{ denom: "uscrt", amount: `${amount}` }]
    );
  }

  /**
   * This method will perform the native token pre_lock.
   * 
   * IMPORTANT: if custom buy token is set, you have to use the SNIP20 
   * receiver callback interface to initiate pre_lock.
   * 
   * @param {number} amount 
   * @param {Agent} [agent] 
   * @returns 
   */
  preLock(amount: string|number|bigint, agent: Agent) {
    return this.tx.pre_lock(
      { amount: `${amount}` },
      agent,
      undefined,
      [{ amount: `${amount}`, denom: "uscrt" }],
    );
  }

  /**
   * Get info about the sale token
   * @return {Promise<object>}
   */
  tokenInfo() {
    return this.q.token_info()
  }

  /**
   * After the sale ends, admin can use this method to refund all tokens that 
   * weren't sold in the IDO sale
   * 
   * @param {string} [address] 
   * @param {Agent} [agent] 
   * @return {Promise<object>}
   */
  adminRefund(address: string, agent: Agent) {
    return this.tx.admin_refund({ address }, agent)
  }

  /**
   * After the sale ends, admin will use this method to claim all the profits 
   * accumulated during the sale
   * 
   * @param {string} [address] 
   * @param {Agent} [agent] 
   * @return {Promise<object>}
   */
  adminClaim(address: string, agent: Agent) {
    return this.tx.admin_claim({ address }, agent)
  }

  /**
   * Add addresses on whitelist for IDO contract
   * 
   * @param {string[]} addresses 
   * @param {Agent} [agent] 
   * @return {Promise<object>}
   */
  adminAddAddresses(addresses: string[], agent: Agent) {
    return this.tx.admin_add_addresses({ addresses }, agent)
  }
}


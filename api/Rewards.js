import { SecretNetwork, loadSchemas } from '@hackbg/fadroma'

export const schema = loadSchemas(import.meta.url, {
    initMsg:     './rewards/init_msg.json',
    queryMsg:    './rewards/query_msg.json',
    queryAnswer: './rewards/query_msg_response.json',
    handleMsg:   './rewards/handle_msg.json'
  })

export default class RewardsContract extends SecretNetwork.Contract.withSchema(schema) {

  get status () { return this.q.status() }
  get admin () { return this.q.admin() }

  getAccounts = (address, lp_tokens, viewing_key) =>
    this.q.accounts({address, lp_tokens, viewing_key})

  simulate = (address, current_time, lp_tokens, viewing_key) =>
    this.q.claim_simulation({address, current_time, lp_tokens, viewing_key})

  lock = (amount, lp_token) =>
    this.tx.lock_tokens({amount, lp_token})

  retrieve = (amount, lp_token) =>
    this.tx.retrieve_tokens({amount, lp_token})

  claim = (lp_tokens) =>
    this.tx.claim({lp_tokens})

  changePools = (pools, total_share) =>
    this.tx.change_pools({pools, total_share})

}

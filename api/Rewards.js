import { SecretNetwork } from '@fadroma/scrt-agent'
import { loadSchemas } from '@fadroma/utilities'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     './rewards/init_msg.json',
  queryMsg:    './rewards/query_msg.json',
  queryAnswer: './rewards/query_msg_response.json',
  handleMsg:   './rewards/handle_msg.json'
})

export default class RewardsContract extends SecretNetwork.Contract.withSchema(schema) {

  get status () {
    return this.q().status()
  }

  get admin () {
    return this.q().admin()
  }

  getAccounts = (address, lp_tokens, viewing_key) =>
    this.q()
      .accounts({ address, lp_tokens, viewing_key })

  simulate = (address, current_time, lp_tokens, viewing_key) =>
    this.q()
      .claim_simulation({ address, current_time, lp_tokens, viewing_key })

  lock = (amount, lp_token, agent) =>
    this.tx(agent)
      .lock_tokens({ amount: String(amount), lp_token })

  retrieve = (amount, lp_token, agent) =>
    this.tx(agent)
      .retrieve_tokens({ amount: String(amount), lp_token })

  claim = (lp_tokens, agent) =>
    this.tx(agent)
      .claim({ lp_tokens })

  changePools = (pools, total_share, agent) =>
    this.tx(agent)
      .change_pools({ pools, total_share })

}

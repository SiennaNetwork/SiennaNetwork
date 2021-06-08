import { SecretNetwork, loadSchemas } from '@fadroma/scrt-agent'

export const schema = loadSchemas(import.meta.url, {
  initMsg:      './lp-staking/l_p_staking_init_msg.json',
  queryMsg:     './lp-staking/l_p_staking_query_msg.json',
  queryAnswer:  './lp-staking/l_p_staking_query_answer.json',
  handleMsg:    './lp-staking/l_p_staking_handle_msg.json'
  handleAnswer: './lp-staking/l_p_staking_handle_answer.json'
})

export default class LPStaking extends SecretNetwork.Contract.withSchema(schema) {}

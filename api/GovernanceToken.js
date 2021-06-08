import { SecretNetwork } from '@fadroma/scrt-agent'
import { loadSchemas } from '@fadroma/utilities'

export const schema = loadSchemas(import.meta.url, {
  initMsg:      './gov-token/init_msg.json',
  queryMsg:     './gov-token/query_msg.json',
  queryAnswer:  './gov-token/query_answer.json',
  handleMsg:    './gov-token/handle_msg.json',
  handleAnswer: './gov-token/handle_answer.json'
})

export default class GovernanceToken extends SecretNetwork.Contract.withSchema(schema) {}

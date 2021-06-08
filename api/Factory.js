import { SecretNetwork, loadSchemas } from '@fadroma/scrt-agent'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     './factory/init_msg.json',
  queryMsg:    './factory/query_msg.json',
  queryAnswer: './factory/query_msg_response.json',
  handleMsg:   './factory/handle_msg.json'
})

export default class Factory extends SecretNetwork.Contract.withSchema(schema) {}

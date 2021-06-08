import { SecretNetwork } from '@fadroma/scrt-agent'
import { loadSchemas } from '@fadroma/utilities'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     './ido/init_msg.json',
  queryMsg:    './ido/query_msg.json',
  queryAnswer: './ido/query_response.json',
  handleMsg:   './ido/handle_msg.json'
})

export default class IDO extends SecretNetwork.Contract.withSchema(schema) {}

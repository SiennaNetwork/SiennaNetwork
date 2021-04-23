import { SecretNetwork, loadSchemas } from '@hackbg/fadroma'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     './cashback-token/init.json',
  queryMsg:    './cashback-token/query.json',
  queryAnswer: './cashback-token/response.json',
  handleMsg:   './cashback-token/handle.json'
})

export default class CashbackToken extends SecretNetwork.Contract.withSchema(schema) {}

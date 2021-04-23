import { SecretNetwork, loadSchemas } from '@hackbg/fadroma'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     './cashback-minter/init.json',
  queryMsg:    './cashback-minter/query.json',
  queryAnswer: './cashback-minter/response.json',
  handleMsg:   './cashback-minter/handle.json'
})

export default class CashbackMinter extends SecretNetwork.Contract.withSchema(schema) {}

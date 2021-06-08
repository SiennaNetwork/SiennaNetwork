import { SecretNetwork, loadSchemas } from '@hackbg/fadroma'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     './weight-master/init.json',
  queryMsg:    './weight-master/query.json',
  queryAnswer: './weight-master/response.json',
  handleMsg:   './weight-master/handle.json'
})

export default class LPStaking extends SecretNetwork.Contract.withSchema(schema) {}

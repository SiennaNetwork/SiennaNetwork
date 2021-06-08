import { SecretNetwork, loadSchemas } from '@hackbg/fadroma'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     './governance-token/init.json',
  queryMsg:    './governance-token/query.json',
  queryAnswer: './governance-token/response.json',
  handleMsg:   './governance-token/handle.json'
})

export default class GovernanceToken extends SecretNetwork.Contract.withSchema(schema) {}

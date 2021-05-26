import { SecretNetwork, loadSchemas } from '@hackbg/fadroma'

export const schema = loadSchemas(import.meta.url, {
    initMsg:     './rewards/init_msg.json',
    queryMsg:    './rewards/query_msg.json',
    queryAnswer: './rewards/query_msg_response.json',
    handleMsg:   './rewards/handle_msg.json'
  })

export default class RewardsContract extends SecretNetwork.Contract.withSchema(schema) { }

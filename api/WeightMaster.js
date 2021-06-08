import { SecretNetwork, loadSchemas } from '@fadroma/scrt-agent'

export const schema = loadSchemas(import.meta.url, {
  initMsg:      './weight-master/master_init_msg.json',
  queryMsg:     './weight-master/master_query_msg.json',
  queryAnswer:  './weight-master/master_query_answer.json',
  handleMsg:    './weight-master/master_handle_msg.json',
  handleAnswer: './weight-master/master_handle_answer.json'
})

export default class WeightMaster extends SecretNetwork.Contract.withSchema(schema) {}

import { SecretNetwork, loadSchemas } from '@hackbg/fadroma'

export const schema = loadSchemas(import.meta.url, {
  initMsg:      './init_msg.json',
  queryMsg:     './query_msg.json',
  queryAnswer:  './query_answer.json',
  handleMsg:    './handle_msg.json',
  handleAnswer: './handle_answer.json'
})

const decoder = new TextDecoder()
const decode = buffer => decoder.decode(buffer).trim()

export default class SNIP20 extends SecretNetwork.Contract.withSchema(schema) {

  setMinters = minters =>
    this.tx.set_minters({minters})

  changeAdmin = address =>
    this.tx.change_admin({address})

  async createViewingKey (agent, entropy = "minimal", address = agent.address) {
    const {data, transactionHash: tx} = await this.tx.create_viewing_key({key}, agent)
    const {key} = JSON.parse(decode(data)).create_viewing_key
    return {tx, key}
  }

  async setViewingKey (agent, key, address = agent.address) {
    const {data, transactionHash: tx} = await this.tx.set_viewing_key({key}, agent)
    const {status} = JSON.parse(decode(data)).set_viewing_key
    return {tx, status}
  }

  async balance (agent, key, address = agent.address) {
    const {balance:{amount}} = await this.q.balance({key, address}, agent)
    return amount
  }

}

import { SecretNetwork, loadSchemas } from '@hackbg/fadroma'

export const schema = loadSchemas(import.meta.url, {
  initMsg:      './token/init_msg.json',
  queryMsg:     './token/query_msg.json',
  queryAnswer:  './token/query_answer.json',
  handleMsg:    './token/handle_msg.json',
  handleAnswer: './token/handle_answer.json'
})

const decoder = new TextDecoder()
const decode = buffer => decoder.decode(buffer).trim()

export default class SNIP20 extends SecretNetwork.Contract.withSchema(schema) {

  static init = (...args) => super.init(...args)

  setMinters = minters =>
    this.tx.set_minters({minters})

  changeAdmin = address =>
    this.tx.change_admin({address})

  async createViewingKey (agent, entropy = "minimal", address = agent.address) {
    const tx = await this.tx.create_viewing_key({key}, agent)
    const {key} = JSON.parse(decode(tx.data)).create_viewing_key
    return {tx, key}
  }

  async setViewingKey (agent, key, address = agent.address) {
    const tx = await this.tx.set_viewing_key({key}, agent)
    const {status} = JSON.parse(decode(tx.data)).set_viewing_key
    return {tx, status}
  }

  async balance (agent, key, address = agent.address) {
    const {balance:{amount}} = await this.q.balance({key, address}, agent)
    return amount
  }

  async mint (agent, amount, recipient, address = agent.address) {
    const tx = await this.tx.mint({amount, recipient, padding: null}, agent)
    const {mint} = JSON.parse(decode(tx.data)).mint
    return {tx, mint}
  }

}

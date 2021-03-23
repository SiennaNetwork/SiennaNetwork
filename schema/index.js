import { SecretNetwork, loadSchemas } from '@hackbg/fadroma'

export const schema = loadSchemas(import.meta.url, {
  initMsg:      './init_msg.json',
  queryMsg:     './query_msg.json',
  queryAnswer:  './query_answer.json',
  handleMsg:    './handle_msg.json',
  handleAnswer: './handle_answer.json'
})

export default class SNIP20 extends SecretNetwork.Contract.withSchema(schema) {

  setMinters = minters =>
    this.tx.set_minters({minters})

  changeAdmin = address =>
    this.tx.change_admin({address})

  createViewingKey = (agent, entropy = "minimal", address = agent.address) =>
    this.tx.create_viewing_key({address, entropy}, agent)
      .then(({data})=>JSON.parse(data).create_viewing_key.key)
      // TODO automatically parse+validate response (in @hackbg/fadroma)

  balance = (agent, key, address = agent.address) =>
    this.q.balance({key, address}, agent)
      .then(({balance:{amount}})=>amount)

}

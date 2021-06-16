import { randomBytes } from 'crypto'
import { SecretNetwork } from '@fadroma/scrt-agent'
import { loadSchemas } from '@fadroma/utilities'

const randomHex = bytes => randomBytes(bytes).toString('hex')

export const schema = loadSchemas(import.meta.url, {
  initMsg:      './snip20/init_msg.json',
  queryMsg:     './snip20/query_msg.json',
  queryAnswer:  './snip20/query_answer.json',
  handleMsg:    './snip20/handle_msg.json',
  handleAnswer: './snip20/handle_answer.json'
})

const decoder = new TextDecoder()
const decode = buffer => decoder.decode(buffer).trim()

export default class SNIP20 extends SecretNetwork.Contract.withSchema(schema) {

  changeAdmin = (address, agent) =>
    this.tx(agent)
      .change_admin({ address })

  setMinters = (minters, agent) =>
    this.tx(agent)
      .set_minters({ minters })

  addMinters = (minters, agent) =>
    this.tx(agent)
      .add_minters({ minters, padding: null })

  mint = (amount, agent = this.agent, recipient = agent.address) =>
    this.tx(agent)
      .mint({ amount: String(amount), recipient, padding: null })
      .then(tx=>({tx, mint: JSON.parse(decode(tx.data)).mint}))

  balance = (address, key) =>
    this.q()
      .balance({ address, key })
      .then(response => response.balance.amount)

  createViewingKey = (agent, entropy = '') =>//randomHex(32)) =>
    this.tx(agent)
      .create_viewing_key({ entropy })
      .then(tx=>({tx, key: JSON.parse(decode(tx.data)).create_viewing_key.key}))

  setViewingKey = (agent, key) =>
    this.tx(agent)
      .set_viewing_key({ key })
      .then(tx=>({tx, status: JSON.parse(decode(tx.data)).set_viewing_key.key}))

  increaseAllowance = (amount, spender, agent) =>
    this.tx(agent)
      .increase_allowance({ amount: String(amount), spender })

  decreaseAllowance = (amount, spender, agent) =>
    this.tx(agent)
      .decrease_allowance({ amount: String(amount), spender })

  checkAllowance = (spender, owner, key, agent) =>
    this.q(agent)
      .allowance({ owner, spender, key })

}

const { SecretNetworkContract } = require('@hackbg/fadroma')

module.exports = class SNIP20 extends SecretNetworkContract.withSchema({
  initMsg:      require('./init_msg.json'),
  queryMsg:     require('./query_msg.json'),
  queryAnswer:  require('./query_answer.json'),
  handleMsg:    require('./handle_msg.json'),
  handleAnswer: require('./handle_answer.json')
}) {

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

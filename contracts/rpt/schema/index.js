const { SecretNetworkContract } = require('@hackbg/fadroma')

module.exports = class RPT extends SecretNetworkContract.withSchema({
  initMsg:     require('./init.json'),
  queryMsg:    require('./query.json'),
  queryAnswer: require('./response.json'),
  handleMsg:   require('./handle.json')
}) {

  // query contract status
  get status () { return this.q.status() }

  // set the splitt proportions
  configure = (config=[]) => this.tx.configure({ config })

  // claim portions from mgmt and distribute them to recipients
  vest = () => this.tx.vest()

}

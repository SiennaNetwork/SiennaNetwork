const { SecretNetworkContract } = require('@hackbg/fadroma')

module.exports = class MGMT extends SecretNetworkContract.withSchema({
  initMsg:     require('./init.json'),
  queryMsg:    require('./query.json'),
  queryAnswer: require('./response.json'),
  handleMsg:   require('./handle.json')
}) {

  // query contract status
  get status () { return this.q.status() }

  // query current schedule
  get schedule () { return this.q.get_schedule() }

  // take over a SNIP20 token
  acquire = async snip20 => {
    await snip20.setMinters([this.address])
    await snip20.changeAdmin(this.address)
  }

  // load a schedule
  configure = schedule => this.tx.configure({ schedule })

  // launch the vesting
  launch = () => this.tx.launch()

  // claim accumulated portions
  claim = claimant => this.tx.claim({}, claimant)

  // add a new account to a pool
  add = (pool, account) => this.tx.add_account({ pool, account })

}

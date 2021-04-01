import { SecretNetwork, loadSchemas } from '@hackbg/fadroma'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     './init.json',
  queryMsg:    './query.json',
  queryAnswer: './response.json',
  handleMsg:   './handle.json'
})

export default class MGMT extends SecretNetwork.Contract.withSchema(schema) {
  // query contract status
  get status () { return this.q.status() }
  // query current schedule
  get schedule () { return this.q.get_schedule() }
  // take over a SNIP20 token
  acquire = async snip20 => {
    await snip20.setMinters([this.address])
    await snip20.changeAdmin(this.address)
    return this
  }
  // load a schedule
  configure = schedule => this.tx.configure({ schedule })
  // launch the vesting
  launch = () =>
    this.tx.launch().then(result=>({
      launched: result.logs[0].events[1].attributes[1].value
    }))
  // claim accumulated portions
  claim = claimant =>
    this.tx.claim({}, claimant)
  // see how much is claimable by someone at a certain time
  progress = (address, time = + new Date()) =>
    this.q.progress({ address, time: Math.floor(time  / 1000) })
  // add a new account to a pool
  add = (pool, account) =>
    this.tx.add_account({ pool, account })
}

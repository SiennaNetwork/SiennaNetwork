module.exports = module.exports.default = class SecretNetworkContract {

  // todo measure gas
  static async deploy ({
    say = require('./say').mute(),
    agent, id, binary,
    name, label, data = {}
  }) {
    say = say.tag(` #${this.name}`)
    if (!id) { // if the contract is not uploaded, do it
      const upload = await agent.upload({ say: say.tag(` #upload`), binary })
      id = upload.codeId
      //await agent.waitForNextBlock()
    }
    const args = say.tag(` #instantiate`)({ id, label, data })
    const {address, hash} = say.tag(` #instantiated`)(await agent.instantiate(args))
    return new this({say, agent, id, binary, name, label, data, address, hash})
  }

  static async fromCommit ({
    say = require('./say').mute(),
    name, commit, binary,
    ...args
  }) {
    say = say.tag(` #${this.name}{${commit}}`)
    const binaryFullPath = require('path').resolve(__dirname, '../../dist/', binary)
    if (!require('fs').existsSync(binaryFullPath)) {
      // * Recompile binary if absent
      say.tag(` #building`)(binaryFullPath)
      const builder = require('path').resolve(__dirname, '../build/commit.sh')
      const build = require('child_process').spawnSync(builder, [ commit ], { stdio: 'inherit' })
      say.tag(` #build-result(${binary})`)(build)
    }
    const label = `${commit} ${name} (${new Date().toISOString()})`
    return this.deploy({say, binary, label, name, ...args})
  }

  constructor (properties = {}) {
    return Object.assign(this, properties)
  }

  async query (method = '', args = {}, agent = this.agent) {
    return await agent.query(this, method, args)
  }

  async execute (method = '', args = {}, agent = this.agent) {
    return await agent.execute(this, method, args)
  }

}

module.exports.SNIP20Contract = class SNIP20Contract extends module.exports {

  static async fromCommit (args={}) {
    args.name   = `TOKEN{${args.commit}}`
    args.binary = `${args.commit}-snip20-reference-impl.wasm`
    args.data   = { name:      "Sienna"
                  , symbol:    "SIENNA"
                  , decimals:  18
                  , admin:     args.agent.address
                  , prng_seed: "insecure"
                  , config:    { public_total_supply: true } }
    return super.fromCommit(args)
  }

  async createViewingKey (agent, address, entropy = "minimal") {
    const response = await agent.execute(this, 'create_viewing_key', { entropy })
    const {create_viewing_key:{key}} = JSON.parse(response.data)
    this.say.tag(` #new-viewing-key`)({'for': address, key})
    return key
  }

  async balance ({ agent, viewkey, address }) {
    const {balance:{amount}} = await this.query('balance', {key: viewkey, address}, agent)
    return amount
  }

  async setMinters (minters = []) {
    return await this.execute('set_minters', {minters})
  }

  async changeAdmin (address) {
    return await this.execute('change_admin', {address})
  }

}

module.exports.MGMTContract = class MGMTContract extends module.exports {

  static async fromCommit (args={}) {
    args.name   = `MGMT{${args.commit}}`
    args.binary = `${args.commit}-sienna-mgmt.wasm`
    args.data   = { token_addr: args.token.address, token_hash: args.token.hash, ...args.data }
    return super.fromCommit(args)
  }

  async acquire (snip20) {
    await snip20.setMinters([this.address])
    await snip20.changeAdmin(this.address)
  }

  async configure (schedule) {
    return await this.execute('configure', {schedule})
  }

  async launch () {
    return await this.execute('launch')
  }

  async claim (claimant) {
    return await claimant.execute(this, 'claim')
  }

  async reallocate (pool_name, channel_name, allocations) {
    return await this.execute('reallocate', { pool_name, channel_name, allocations })
  }

  async addChannel () { /* TODO */ }

}

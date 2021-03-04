module.exports = module.exports.default = class SecretNetworkContract {

  // todo measure gas
  static async deploy ({
    say = require('./say').tag(`#${this.name}`),
    agent, id, binary,
    name, label, data = {}
  }) {
    if (!id) { // if the contract is not uploaded, do it
      id = await agent.upload({
        say: say.tag(` #upload`),
        binary
      })
      //await agent.waitForNextBlock()
    }
    const args = say.tag(` #instantiate`)({ id, label, data })
    const {address, hash} = say.tag(` #instantiated`)(await agent.instantiate(args))
    return new this({ say, agent, id, binary, name, label, data, address, hash })
  }

  static async fromCommit ({
    name, commit, binary,
    say = require('./say').tag(`${this.name}{${commit}}`),
    ...args
  }) {
    if (!require('fs').existsSync(binary)) {
      // * Recompile binary if absent
      say.tag(` #building`)(binary)
      const builder = require('path').resolve(__dirname, '../build/commit.sh')
      const build = require('child_process').spawnSync(builder, [ commit ], { stdio: 'inherit' })
      say.tag(` #build-result(${binary})`)(build)
    }
    const label = `${commit} ${name} (${new Date().toISOString()})`
    return this.deploy({binary, label, name, ...args})
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
    args.binary = require('path').resolve(__dirname, `../../dist/${args.commit}-snip20-reference-impl.wasm`)
    args.data   = { name:      "Sienna"
                  , symbol:    "SIENNA"
                  , decimals:  18
                  , admin:     args.agent.address
                  , prng_seed: "insecure"
                  , config:    { public_total_supply: true } }
    return super.fromCommit(args)
  }

  async createViewingKey (agent, address, entropy = "minimal") {
    const response = await agent.execute(address, 'create_viewing_key', { entropy })
    const {create_viewing_key:{key}} = JSON.parse(response.data)
    this.say.tag(` #new-viewing-key`)({'for': address, key})
    return key
  }

  async balance ({ agent, viewkey, address }) {
    return await this.query('balance', {key: viewkey, address}, agent)
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
    args.binary = require('path').resolve(__dirname, `../../dist/${args.commit}-sienna-mgmt.wasm`)
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
    try {
      return await claimant.execute(this, 'claim')
    } catch (error) {
      return this.say.tag(` #${this.name} #error`)(error)
    }
  }

  async addChannel () {}

  async reallocate () {}

}

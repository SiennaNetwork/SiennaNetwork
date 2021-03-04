const $ = module.exports = {

  async getBalance ({
    say   = require('./say')('[getBalance]'),
    key   = $.createViewingKey({agent, token}),
    agent = require('./agent').fromEnvironment(),
    token,
    address
  }={}) {
    key = await Promise.resolve(key)
    return say(await agent.query(token.address, { balance: { key, address } }))
  },

  async createViewingKey ({
    say   = require('./say')('[createViewingKey]'),
    agent = require('./agent').fromEnvironment(),
    token,
    entropy = "minimal"
  }={}) {
    const method = 'create_viewing_key'
    const {[method]:{key}} = JSON.parse(require("@iov/encoding").fromUtf8(
      await agent.execute(token.address,{[method]:{entropy}})
    ))
    return say(key)
  }

}

module.exports = module.exports.default = class SecretNetworkAgent {

  // the API endpoint

  static APIURL = require('./say').tag('APIURL')(
    process.env.SECRET_REST_URL || 'http://localhost:1337')

  // ways of creating authenticated clients

  static async fromKeyPair ({
    say     = require('./say'),
    name    = "",
    keyPair = require('secretjs').EnigmaUtils.GenerateNewKeyPair()
  }={}) {
    const mnemonic = require('@cosmjs/crypto').Bip39.encode(keyPair.privkey).data
    return await SecretNetworkAgent.fromMnemonic({name, mnemonic, keyPair, say})
  }

  static async fromMnemonic ({
    say      = require('./say'),
    name     = "",
    mnemonic = process.env.MNEMONIC,
    keyPair // optional
  }={}) {
    const pen = await require('secretjs').Secp256k1Pen.fromMnemonic(mnemonic)
    return new SecretNetworkAgent({name, pen, keyPair, say, mnemonic})
  }

  // initial setup

  constructor ({
    say  = require('./say'),
    name = "",
    pen,
    keyPair,
    mnemonic,
    fees = require('./gas').defaultFees,
    secretjs: { encodeSecp256k1Pubkey, pubkeyToAddress, EnigmaUtils, SigningCosmWasmClient
              } = require('secretjs')
  }) {
    Object.assign(this, {
      name, keyPair, pen, mnemonic, fees,
      say: say.tag(`@${name}`)
    })
    this.pubkey  = encodeSecp256k1Pubkey(this.pen.pubkey)
    this.address = pubkeyToAddress(this.pubkey, 'secret')
    this.seed    = EnigmaUtils.GenerateNewSeed()
    this.sign    = pen.sign.bind(pen)
    this.API     = new (require('secretjs').SigningCosmWasmClient)(
      SecretNetworkAgent.APIURL, this.address, this.sign, this.seed, this.fees)
    return this
  }

  // interact with the network:

  async status () {
    const {header:{time,height}} = await this.API.getBlock()
    return this.say.tag(' #status')({
      time,
      height,
      account: await this.API.getAccount(this.address)
    })
  }

  async account () {
    const {execFileSync} = require('child_process')
    const account = JSON.parse(execFileSync('secretcli', [ 'query', 'account', this.address ]))
    return this.say.tag(` #account`)(account)
  }

  async time () {
    const {header:{time,height}} = await this.API.getBlock()
    return this.say.tag(' #time')({time,height})
  }

  async waitForNextBlock () {
    const {header:{height}} = await this.API.getBlock()
    this.say('waiting for next block before continuing...')
    while (true) {
      await new Promise(ok=>setTimeout(ok, 1000))
      const now = await this.API.getBlock()
      if (now.header.height > height) break
    }
  }

  async query ({ name, address }, method='', args={}) {
    this.say.tag(` #${name} #${method}?`)(args)
    const response = await this.API.queryContractSmart(address, {[method]:args})
    this.say.tag(` #${name} #${method}? #returned`)(response)
    return response
  }

  async execute ({ name, address }, method='', args={}) {
    this.say.tag(` #${name} #${method}!`)(args)
    const response = await this.API.execute(address, {[method]:args})
    this.say.tag(` #${name} #${method}! #returned`)(response)
    return response
  }

  // deploy smart contracts to the network:

  async upload ({
    say=this.say,
    binary
  }) { // upload code to public registry
    const {resolve} = require('path')
    binary = resolve(resolve(__dirname, '..'), binary)
    say(binary)
    const wasm = await require('fs').promises.readFile(binary)
    const uploadReceipt = await this.API.upload(wasm, {});
    say(uploadReceipt)
    return uploadReceipt.codeId
  }

  async instantiate ({
    id, data = {}, label = ''
  }) { // initial transaction
    const {contractAddress} = await this.API.instantiate(id, data, label)
    // TODO get contract hash from secretjs
    const {execFileSync} = require('child_process')
    const hash = execFileSync('secretcli', [ 'query', 'compute', 'contract-hash', contractAddress ])
    return {
      id, label,
      address: contractAddress,
      hash:    String(hash).slice(2)
    }
  }

}

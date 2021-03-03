const {resolve} = require('path')
const {execFileSync} = require('child_process')
const { Secp256k1Pen, encodeSecp256k1Pubkey
      , pubkeyToAddress
      , EnigmaUtils: { GenerateNewSeed }
      , SigningCosmWasmClient, } = require('secretjs')

module.exports = {
  fromEnvironment: getAgentFromEnvironment,
  fromKeyPair:     getAgentFromKeyPair
}

async function getAgentFromKeyPair (name, {
  say  = require('./say')(`[agent ${name}]`),
  url  = process.env.SECRET_REST_URL || 'http://localhost:1337',
  key  = require('secretjs').EnigmaUtils.GenerateNewKeyPair(),
  fees = require('./gas').defaultFees,
} = {}) {
  const {privkey, pubkey} = key
  const mne = require('@cosmjs/crypto').Bip39.encode(privkey).data
  const {API, addr} = await getAPI(url, mne, fees)
  say(mne)
  return Object.assign(agentMethods(API, say), { name, addr, privkey, pubkey, mnemonic: mne })
}

async function getAgentFromEnvironment ({
  say  = require('./say')("[agent ADMIN]"),
  env  = require('./env')(),
  url  = process.env.SECRET_REST_URL || 'http://localhost:1337',
  mne  = process.env.MNEMONIC,
  fees = require('./gas').defaultFees,
} = {}) {
  const {API, addr, pubkey} = await getAPI(url, mne, fees)
  return Object.assign(agentMethods(API, say), { name: 'env', addr, pubkey, mnemonic: mne })
}

async function getAPI (url, mne, fees) {
  const pen  = await Secp256k1Pen.fromMnemonic(mne);
  const pub  = encodeSecp256k1Pubkey(pen.pubkey);
  const addr = pubkeyToAddress(pub, 'secret');
  const seed = GenerateNewSeed();
  const API  = new SigningCosmWasmClient(
    url,
    addr,
    pen.sign.bind(pen),
    seed,
    fees
  )
  return {API, addr, pubkey: pub}
}

function agentMethods (API, say) {
  return {
    async status () {
      const {header:{time,height}} = await API.getBlock()
      return 
    },
    async deploy (source, label, data = {}) {
      // todo measure gas
      const id = await this.upload(source)
      return await this.init(id, label, data)
    },
    async upload (source) {
      source = resolve(resolve(__dirname, '..'), source)
      say(`uploading ${source}`)
      const wasm = await require('fs').promises.readFile(source)
      const uploadReceipt = await API.upload(wasm, {});
      return uploadReceipt.codeId
    },
    async init (id, label, data = {}) {
      say(`init ${id} as ${label} with ${JSON.stringify(data)}`)
      const {contractAddress} = await API.instantiate(id, data, label)
      const hash = execFileSync('secretcli', [
        'query', 'compute', 'contract-hash', contractAddress
      ])
      return {
        id,
        label,
        address: contractAddress,
        hash: String(hash).slice(2)
      }
    },
    async query (addr, msg={}) {
      return await API.queryContractSmart(addr, msg)
    },
    async execute (addr, msg={}) {
      return await API.execute(addr, msg)
    },
  }
}

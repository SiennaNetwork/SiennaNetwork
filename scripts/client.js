const {resolve} = require('path')
const {execFileSync} = require('child_process')
const { Secp256k1Pen, encodeSecp256k1Pubkey
      , pubkeyToAddress
      , EnigmaUtils: { GenerateNewSeed }
      , SigningCosmWasmClient, } = require('secretjs')

module.exports = async function getClient ({
  env  = require('./env')(),
  url  = process.env.SECRET_REST_URL || 'http://localhost:1337',
  mne  = process.env.MNEMONIC,
  fees = require('./gas').defaultFees,
  say = require('./say')("[client]")
} = {}) {

  const pen  = await Secp256k1Pen.fromMnemonic(mne);
  const pub  = encodeSecp256k1Pubkey(pen.pubkey);
  const addr = pubkeyToAddress(pub, 'secret');
  const seed = GenerateNewSeed();
  const sign = b => pen.sign(b)
  const API  = new SigningCosmWasmClient(url, addr, sign, seed, fees);

  return {
    API,
    async deploy (source, label, data = {}) {
      const id = await upload(source)
      return await init(id, label, data)
    },
    async query (addr, msg={}) {
      return await API.queryContractSmart(addr, msg)
    },
    async execute (addr, msg={}) {
      return await API.execute(addr, msg)
    },
    async status () {
      const {header:{time,height}} = await API.getBlock()
      return 
    }
  }

  async function upload (source) {
    source = resolve(resolve(__dirname, '..'), source)
    say(`uploading ${source}`)
    const wasm = await require('fs').promises.readFile(source)
    const uploadReceipt = await API.upload(wasm, {});
    return uploadReceipt.codeId
  }

  async function init (id, label, data = {}) {
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
  }

}

const {
  Secp256k1Pen, encodeSecp256k1Pubkey,
  pubkeyToAddress,
  EnigmaUtils: { GenerateNewSeed },
  SigningCosmWasmClient,
} = require('secretjs');

const gas = require('./gas')

module.exports = async function getClient (
  url =
    process.env.SECRET_REST_URL || 'http://localhost:1337',
  mnemonic =
    process.env.MNEMONIC,
  fees =
    { upload: gas(3000000)
    , init:   gas( 500000)
    , exec:   gas( 500000)
    , send:   gas(  80000) }
) {
  const pen = await Secp256k1Pen.fromMnemonic(mnemonic);
  const pub = encodeSecp256k1Pubkey(pen.pubkey);
  const addr = pubkeyToAddress(pub, 'secret');
  const seed = GenerateNewSeed();
  const sign = b => pen.sign(b)
  const client = new SigningCosmWasmClient(url, addr, sign, seed, fees);

  return { client, deploy, query, execute }

  async function deploy (source, label, data = {}) {
    const id = await upload(source)
    return await init(id, label, data)
  }
  async function upload (source) {
    const wasm = await require('fs').promises.readFile(source)
    const uploadReceipt = await client.upload(wasm, {});
    return uploadReceipt.codeId
  }
  async function init (id, label, data = {}) {
    const {contractAddress} = await client.instantiate(id, data, label)
    const hash = require('child_process').execFileSync('secretcli',
      ['query', 'compute', 'contract-hash', contractAddress])
    return {
      id,
      label,
      address: contractAddress,
      hash: String(hash).slice(2)
    }
  }

  async function query (addr, msg={}) {
    return await client.queryContractSmart(addr, msg)
  }

  async function execute (addr, msg={}) {
    return await client.execute(addr, msg)
  }
}

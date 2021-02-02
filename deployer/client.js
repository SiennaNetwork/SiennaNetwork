const {
  Secp256k1Pen, encodeSecp256k1Pubkey,
  pubkeyToAddress,
  EnigmaUtils: { GenerateNewSeed },
  SigningCosmWasmClient,
} = require('secretjs');

module.exports = async function getClient (url, mnemonic, fees) {
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
    return {address: contractAddress, hash: String(hash)}
  }

  async function query (addr, msg={}) {
    return await client.queryContractSmart(addr, msg)
  }

  async function execute (addr, msg={}) {
    return await client.execute(addr, msg)
  }
}

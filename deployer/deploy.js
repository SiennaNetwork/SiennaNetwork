#!/usr/bin/env node

const {
  Secp256k1Pen, encodeSecp256k1Pubkey,
  pubkeyToAddress,
  EnigmaUtils: { GenerateNewSeed },
  SigningCosmWasmClient,
} = require('secretjs');

process.on('unhandledRejection', up => {throw up})
require('dotenv').config()

module.exports = main
if (require.main === module) main()

async function main (
  httpUrl  = process.env.SECRET_REST_URL,
  mnemonic = process.env.MNEMONIC,
  customFees =
    { upload: { amount: [{ amount: '2000000', denom: 'uscrt' }], gas: '2000000' }
    , init:   { amount: [{ amount:  '500000', denom: 'uscrt' }], gas:  '500000' }
    , exec:   { amount: [{ amount:  '500000', denom: 'uscrt' }], gas:  '500000' }
    , send:   { amount: [{ amount:   '80000', denom: 'uscrt' }], gas:   '80000' } }
) {

  const client = await getClient(httpUrl, mnemonic, customFees)

  const token = await client.deploy(
    `${__dirname}/../dist/snip20-reference-impl.wasm.gz`,
    `SIENNA SNIP20 (${+new Date().toISOString()})`, {
      name:      "Sienna",
      symbol:    "SIENNA",
      decimals:  18,
      admin:     client.address,
      prng_seed: "insecure",
      config:    { public_total_supply: true }
    })

  const mgmt = await client.deploy(
    `${__dirname}/../dist/sienna-mgmt.wasm.gz`,
    `SIENNA MGMT (${+new Date().toISOString()})`, {
      token_addr: token,
      token_hash: ""
    })

}

async function getClient (url, mnemonic, fees) {
  const pen = await Secp256k1Pen.fromMnemonic(mnemonic);
  const pub = encodeSecp256k1Pubkey(pen.pubkey);
  const addr = pubkeyToAddress(pub, 'secret');
  const seed = GenerateNewSeed();
  const sign = b => pen.sign(b)
  const client = new SigningCosmWasmClient(url, addr, sign, seed, fees);

  return { client, deploy, query, execute }

  async function deploy (source, label, data = {}) {
    const id = await upload(source)
    const addr = await init(id, label, data)
    addr
  }
  async function upload (source) {
    const wasm = await require('fs').promises.readFile(source)
    const uploadReceipt = await client.upload(wasm, {});
    return uploadReceipt.codeId
  }
  async function init (id, label, data = {}) {
    const contract = await client.instantiate(id, data, label)
    return contract.contractAddress
  }

  async function query (addr, msg={}) {
    return await client.queryContractSmart(addr, msg)
  }

  async function execute (addr, msg={}) {
    return await client.execute(addr, msg)
  }
}

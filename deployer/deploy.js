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
  httpUrl  = process.env.SECRET_REST_URL || 'http://localhost:1337',
  mnemonic = process.env.MNEMONIC || 'cloth pig april pitch topic column festival vital plate spread jewel twin where crouch leader muscle city brief jacket elder ritual loop upper place',
  customFees =
    { upload: { amount: [{ amount: '3000000', denom: 'uscrt' }], gas: '3000000' }
    , init:   { amount: [{ amount:  '500000', denom: 'uscrt' }], gas:  '500000' }
    , exec:   { amount: [{ amount:  '500000', denom: 'uscrt' }], gas:  '500000' }
    , send:   { amount: [{ amount:   '80000', denom: 'uscrt' }], gas:   '80000' } }
) {

  const client = await getClient(httpUrl, mnemonic, customFees)

  console.log('deploying token...')
  const token = await client.deploy(
    `${__dirname}/../dist/snip20-reference-impl.wasm`,
    `SIENNA SNIP20 (${new Date().toISOString()})`, {
      name:      "Sienna",
      symbol:    "SIENNA",
      decimals:  18,
      admin:     client.address,
      prng_seed: "insecure",
      config:    { public_total_supply: true }
    })
  console.log(token)

  console.log('deploying mgmt...')
  const mgmt = await client.deploy(
    `${__dirname}/../dist/sienna-mgmt.wasm`,
    `SIENNA MGMT (${new Date().toISOString()})`, {
      token_addr: token.address,
      token_hash: token.hash
    })
  console.log(mgmt)

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

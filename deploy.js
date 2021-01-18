#!/usr/bin/env node

const {
  EnigmaUtils,
  Secp256k1Pen,
  SigningCosmWasmClient,
  pubkeyToAddress,
  encodeSecp256k1Pubkey,
} = require('secretjs');

process.on('unhandledRejection', up => {throw up})
require('dotenv').config()

module.exports = main
if (require.main === module) main()

async function main (
  httpUrl  = process.env.SECRET_REST_URL,
  mnemonic = process.env.MNEMONIC,
  customFees = {
    upload: { amount: [{ amount: '2000000', denom: 'uscrt' }], gas: '2000000' },
    init:   { amount: [{ amount:  '500000', denom: 'uscrt' }], gas:  '500000' },
    exec:   { amount: [{ amount:  '500000', denom: 'uscrt' }], gas:  '500000' },
    send:   { amount: [{ amount:   '80000', denom: 'uscrt' }], gas:   '80000' },
  }
) {
  const signingPen = await Secp256k1Pen.fromMnemonic(mnemonic);
  const pubkey = encodeSecp256k1Pubkey(signingPen.pubkey);
  const accAddress = pubkeyToAddress(pubkey, 'secret');
  const txEncryptionSeed = EnigmaUtils.GenerateNewSeed();
  const client = new SigningCosmWasmClient(
    httpUrl, accAddress,
    (signBytes) => signingPen.sign(signBytes),
    txEncryptionSeed, customFees,
  );
  console.log('Wallet address: ', accAddress);

  //const buildTarget = 'wasm32-unknown-unknown'
  //const crateName = 'scrt_calc'
  //const wasmPath = `${__dirname}/target/${buildTarget}/release/${crateName}.wasm`
  const wasmPath = `${__dirname}/contract.wasm`
  const wasm = require('fs').readFileSync(wasmPath)
  const uploadReceipt = await client.upload(wasm, {});
  const contract = await client.instantiate(
    uploadReceipt.codeId,
    { value: 101 },
    'My Calculator' + Math.ceil(Math.random() * 10000),
  );
  const contractAddress = contract.contractAddress;
  console.log('contract:', contract);

  let response = await client.queryContractSmart(contractAddress, { equals: {}, })
  console.log('Equals: ', response.value)

  for (let operation of [
    { add: { augend:        1 } },
    { sub: { subtrahend:   10 } },
    { mul: { multiplier:  100 } },
    { div: { divisor:    1000 } }
  ]) {
    let response = await client.execute(contractAddress, operation)
    console.log('Response to', operation, ': ', response)
    response = await client.queryContractSmart(contractAddress, { equals: {}, })
    console.log('Equals: ', response.value)
  }
}

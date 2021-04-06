import { FactoryContract, ContractInstantiationInfo, ContractInfo } from './types.js'
import { SecretNetwork, say as sayer } from '@hackbg/fadroma'
import { SigningCosmWasmClient, Secp256k1Pen, encodeSecp256k1Pubkey, EnigmaUtils, pubkeyToAddress, CosmWasmClient } from 'secretjs'
import { Bip39, Random } from "@iov/crypto"

import { resolve } from 'path'
import.meta.url

interface LocalAccount {
  name: string,
  type: string,
  address: string,
  pubkey: string,
  mnemonic: string
}

const APIURL = 'http://localhost:1337'

const ACC: object[] = JSON.parse(process.argv[3])
const ACC_A: LocalAccount = ACC[0] as LocalAccount
const ACC_B: LocalAccount = ACC[1] as LocalAccount
const ACC_C: LocalAccount = ACC[2] as LocalAccount
const ACC_D: LocalAccount = ACC[3] as LocalAccount

const say = sayer.tag(() => new Date().toISOString())

const FEES = {
  upload: {
      amount: [{ amount: "2000000", denom: "uscrt" }],
      gas: "2000000",
  },
  init: {
      amount: [{ amount: "500000", denom: "uscrt" }],
      gas: "500000",
  },
  exec: {
      amount: [{ amount: "500000", denom: "uscrt" }],
      gas: "500000",
  },
  send: {
      amount: [{ amount: "80000", denom: "uscrt" }],
      gas: "80000",
  },
}

async function run_tests() {
  
  
  console.log(`Acc: ${ACC_A.mnemonic}`)
}

async function setup() {
  const commit = process.argv[2]

  SecretNetwork.Agent.APIURL = 'http://localhost:1337'

  const snip20_wasm = resolve(`../dist/${commit}-snip20-reference-impl.wasm`)
  const exchange_wasm = resolve(`../dist/${commit}-exchange.wasm`)
  const ido_wasm = resolve(`../dist/${commit}-ido.wasm`)

  const client = await SecretNetwork.Agent.fromKeyPair({say, name: "test-client"})
  const builder = new SecretNetwork.Builder({ say: say.tag('builder'), outputDir: '', agent: client })

  const exchange_upload = await builder.upload(exchange_wasm)
  const snip20_token_upload = await builder.upload(snip20_wasm)
  const ido_upload = await builder.upload(ido_wasm)

  const exchange_contract_info = new ContractInstantiationInfo(exchange_upload.transactionHash, exchange_upload.codeId)
  const snip20_token_contract_info = new ContractInstantiationInfo(snip20_token_upload.transactionHash, snip20_token_upload.codeId)
  const ido_contract_info = new ContractInstantiationInfo(ido_upload.transactionHash, ido_upload.codeId)
  const sienna_token = new ContractInfo("test", "test")
  
  //const factory = await FactoryContract.instantiate(say, commit, snip20_token_contract_info, exchange_contract_info, ido_contract_info, sienna_token)

  //return { client, factory }
}

run_tests().catch(console.log)

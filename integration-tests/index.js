import { FactoryContract, ContractInstantiationInfo, ContractInfo } from './contract.js'
import { say as sayer, SecretNetwork } from '@hackbg/fadroma'

import { resolve } from 'path'
import.meta.url

const say = sayer.tag(() => new Date().toISOString())

async function run_tests() {
    const { client, factory } = await setup()

    console.log(await factory.get_exchange_pair("invalid address"))
}

async function setup() {
  const commit = process.argv[2]

  SecretNetwork.Agent.APIURL = 'http://localhost:1337';

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

  const factory = await FactoryContract.instantiate(say, commit, snip20_token_contract_info, exchange_contract_info, ido_contract_info, sienna_token)

  return { client, factory }
}

run_tests().then(console.log)

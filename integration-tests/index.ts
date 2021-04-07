import { 
  ContractInstantiationInfo, ContractInfo, TokenPair
} from './amm-lib/types.js'
import { FactoryContract, FEES } from './amm-lib/contract.js'
import { 
  SigningCosmWasmClient, Secp256k1Pen, encodeSecp256k1Pubkey,
  EnigmaUtils, pubkeyToAddress
} from 'secretjs'
import { Bip39, Random } from "@iov/crypto"

import { resolve } from 'path'
import { readFileSync } from 'fs'
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

interface SetupResult {
  factory: FactoryContract,
  sienna_token: ContractInfo
}

interface AsyncFn {
  (): Promise<void>
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))
const SLEEP_TIME = 1000

async function run_tests() {
  const client_a = await build_client(ACC_A.mnemonic)
  const { factory, sienna_token } = await setup(client_a)

  const created_pair = await test_create_exchange(factory, sienna_token)
  await sleep(SLEEP_TIME)
  await test_create_existing_pair_error(factory, created_pair)
}

async function setup(client: SigningCosmWasmClient): Promise<SetupResult> {
  const commit = process.argv[2]

  const snip20_wasm = readFileSync(resolve(`../dist/${commit}-snip20-reference-impl.wasm`))
  const exchange_wasm = readFileSync(resolve(`../dist/${commit}-exchange.wasm`))
  const ido_wasm = readFileSync(resolve(`../dist/${commit}-ido.wasm`))
  const factory_wasm = readFileSync(resolve(`../dist/${commit}-factory.wasm`))

  const exchange_upload = await client.upload(exchange_wasm, {})
  const snip20_upload = await client.upload(snip20_wasm, {})
  const ido_upload = await client.upload(ido_wasm, {})
  const factory_upload = await client.upload(factory_wasm, {})

  const pair_contract = new ContractInstantiationInfo(exchange_upload.originalChecksum, exchange_upload.codeId)
  const snip20_contract = new ContractInstantiationInfo(snip20_upload.originalChecksum, snip20_upload.codeId)
  const ido_contract = new ContractInstantiationInfo(ido_upload.originalChecksum, ido_upload.codeId)

  const sienna_init_msg = {
    name: 'sienna',
    symbol: 'SIENNA',
    decimals: 18,
    prng_seed: 'MTMyMWRhc2RhZA=='
  } 

  const sienna_contract = await client.instantiate(snip20_upload.codeId, sienna_init_msg, 'SIENNA TOKEN')
  const sienna_token = new ContractInfo(snip20_upload.originalChecksum, sienna_contract.contractAddress)

  const factory_init_msg = {
    snip20_contract,
    pair_contract,
    ido_contract,
    sienna_token
  }
  
  const result = await client.instantiate(factory_upload.codeId, factory_init_msg, 'AMM-FACTORY')
  const factory = new FactoryContract(client, result.contractAddress)

  return { factory, sienna_token }
}

async function build_client(mnemonic: string): Promise<SigningCosmWasmClient> {
  const pen = await Secp256k1Pen.fromMnemonic(mnemonic)
  const seed = EnigmaUtils.GenerateNewSeed();

  const pubkey  = encodeSecp256k1Pubkey(pen.pubkey)
  const address = pubkeyToAddress(pubkey, 'secret')

  return new SigningCosmWasmClient(
    APIURL,
    address,
    (bytes) => pen.sign(bytes),
    seed,
    FEES
  )
}

async function test_create_exchange(factory: FactoryContract, token_info: ContractInfo): Promise<TokenPair> {
  const pair = new TokenPair({
      native_token: {
        denom: 'uscrt'
      }
    },{
      custom_token: {
        contract_addr: token_info.address,
        token_code_hash: token_info.code_hash
      }
    }
  )
  
  await execute_test(
    'test_create_exchange',
    async () => { await factory.create_exchange(pair); }
  )

  return pair
}

async function test_create_existing_pair_error(factory: FactoryContract, pair: TokenPair) {
  await execute_test_expect(
    'test_create_existing_pair_error',
    async () => { await factory.create_exchange(pair) },
    'Pair already exists'
  )

  await sleep(SLEEP_TIME)

  const swapped = new TokenPair(pair.token_1, pair.token_0)

  await execute_test_expect(
    'test_create_existing_pair_error_swapped',
    async () => { await factory.create_exchange(swapped) },
    'Pair already exists'
  )
}

async function execute_test(test_name: string, test: AsyncFn) {
  try {
    await test()
    print_success(test_name)
  } catch(e) {
    console.error(e)
    print_error(test_name)
  }
}

async function execute_test_expect(
    test_name: string,
    test: AsyncFn,
    expected_error: string
) {
  try {
    await test()
    print_error(`${test_name}(expected error)`)
  } catch (e) {
    if (e.message.includes(expected_error)) {
      print_success(test_name)
      return
    }

    console.error(e)
    print_error(test_name)
  }
}

function print_success(test_name: string) {
  console.log(`${test_name}..............................✅`)
}

function print_error(test_name: string) {
  console.log(`${test_name}..............................❌`)
}

run_tests().catch(console.log)

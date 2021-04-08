import { 
  ContractInstantiationInfo, ContractInfo,
  TokenPair, Address, TokenPairAmount, ViewingKey
} from './amm-lib/types.js'
import { FactoryContract, ExchangeContract, Snip20Contract, FEES } from './amm-lib/contract.js'
import { 
  execute_test, execute_test_expect, assert_objects_equal,
  assert_equal, assert_not_equal
} from './test_helpers.js'
import { 
  SigningCosmWasmClient, Secp256k1Pen, encodeSecp256k1Pubkey,
  EnigmaUtils, pubkeyToAddress
} from 'secretjs'
import { Sha256, Random } from "@iov/crypto"
import { Buffer } from 'buffer'

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

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))
const SLEEP_TIME = 1000

async function run_tests() {
  const client_a = await build_client(ACC_A.mnemonic)
  const { factory, sienna_token } = await setup(client_a)

  const created_pair = await test_create_exchange(factory, sienna_token)
  await sleep(SLEEP_TIME)

  await test_create_existing_pair_error(factory, created_pair)
  
  const pair_address = await test_get_exchange_address(factory, created_pair)
  await test_get_exchange_pair(factory, created_pair, pair_address)

  const exchange = new ExchangeContract(client_a, pair_address)
  await test_get_pair_info(exchange, created_pair)
  await test_get_factory_info(exchange, factory.address)
  await test_get_pool(exchange)

  const snip20 = new Snip20Contract(client_a, sienna_token.address)

  await test_iquidity(exchange, snip20, created_pair)
  await sleep(SLEEP_TIME)
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
    async () => { await factory.create_exchange(pair) }
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

async function test_get_exchange_address(factory: FactoryContract, pair: TokenPair): Promise<Address> {
  let address = '';

  await execute_test(
    'test_get_exchange_address',
    async () => { 
      const result = await factory.get_exchange_address(pair)
      address = result
    }
  )

  return address
}

async function test_get_exchange_pair(factory: FactoryContract, pair: TokenPair, address: Address) {
  await execute_test(
    'test_get_exchange_pair',
    async () => { 
      const result = await factory.get_exchange_pair(address)
      assert_objects_equal(pair, result)
    }
  )
}

async function test_get_pair_info(exchange: ExchangeContract, pair: TokenPair) {
  await execute_test(
    'test_get_pair_info',
    async () => {
      const result = await exchange.get_pair_info()
      assert_objects_equal(pair, result)
    }
  )
}

async function test_get_factory_info(exchange: ExchangeContract, address: Address) {
  await execute_test(
    'test_get_factory_info',
    async () => {
      const result = await exchange.get_factory_info()
      assert_equal(address, result.address)
    }
  )
}

async function test_get_pool(exchange: ExchangeContract) {
  await execute_test(
    'test_get_pool',
    async () => {
      const result = await exchange.get_pool()
      assert_equal(result.amount_0, '0')
      assert_equal(result.amount_1, '0')
    }
  )
}

async function test_iquidity(exchange: ExchangeContract, snip20: Snip20Contract, pair: TokenPair) {
  const amount = '5000000'

  // TODO: The current snip20 implementation is garbage and doesn't implement
  // decimal conversion, so providing only a single amount for now
  //const amount1 = '5000000000000000000'

  await snip20.deposit(amount)
  await sleep(SLEEP_TIME)

  await snip20.increase_allowance(exchange.address, amount)
  await sleep(SLEEP_TIME)

  const token_amount = new TokenPairAmount(pair, amount, amount) // 5 of each

  await execute_test(
    'test_provide_liquidity',
    async () => {
      const result = await exchange.provide_liquidity(token_amount)
      assert_equal(result.logs[0].events[2].attributes[3].value, amount) //LP tokens
    }
  )

  await execute_test(
    'test_provide_liquidity_pool_not_empty',
    async () => {
      const result = await exchange.get_pool()
      assert_equal(result.amount_0, amount)
      assert_equal(result.amount_1, amount)
    }
  )

  await sleep(SLEEP_TIME)

  await execute_test(
    'test_withdraw_liquidity',
    async () => {
      const result = await exchange.withdraw_liquidity(amount, exchange.client.senderAddress)
      console.log(JSON.stringify(result.logs, null, 2))
    }
  )
}

export function create_viewing_key(): ViewingKey {
  const rand_bytes = Random.getBytes(32)
  const key = new Sha256(rand_bytes).digest()

  return Buffer.from(key).toString('base64')
}

run_tests().catch(console.log)

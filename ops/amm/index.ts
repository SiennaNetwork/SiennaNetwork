import { 
  ContractInfo, TokenPair, Address, TokenPairAmount,
  ViewingKey, Uint128, TokenTypeAmount, Pagination, ContractInstantiationInfo, ExchangeSettings
} from './amm-lib/types.js'
import { FactoryContract, ExchangeContract, Snip20Contract, create_fee } from './amm-lib/contract.js'
import { 
  execute_test, execute_test_expect, assert_objects_equal, assert,
  assert_equal, assert_not_equal, extract_log_value, print_object
} from './utils/test_helpers.js'
import { upload_amm, build_client, UploadResult, create_rand_base64 } from './setup.js'
import { NullJsonFileWriter } from './utils/json_file_writer.js'
import { TxAnalytics } from './utils/tx_analytics.js'
import { SigningCosmWasmClient, Account } from 'secretjs'
import { Sha256, Random } from "@iov/crypto"
import { Buffer } from 'buffer'
import { table } from 'table';

import.meta.url

interface LocalAccount {
  name: string,
  type: string,
  address: string,
  pubkey: string,
  mnemonic: string
}

const APIURL = 'http://localhost:1337'

const ACC: object[] = JSON.parse(process.argv[2])
const ACC_A: LocalAccount = ACC[0] as LocalAccount
const ACC_B: LocalAccount = ACC[1] as LocalAccount
const ACC_C: LocalAccount = ACC[2] as LocalAccount
const BURN_POOL: LocalAccount = ACC[3] as LocalAccount

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))
const SLEEP_TIME = 1000

const analytics: TxAnalytics = new TxAnalytics(APIURL)

async function run_tests() {
  const client_a = await build_client(ACC_A.mnemonic, APIURL)
  
  const result = await upload_amm(client_a, new NullJsonFileWriter)

  const factory = await instantiate_factory(client_a, result)
  const sienna_token = await instantiate_sienna_token(client_a, result.snip20)

  const created_pair = await test_create_exchange(factory, sienna_token)
  await sleep(SLEEP_TIME)

  await test_create_existing_pair_error(factory, created_pair)
  
  const pair_address = await test_query_exchanges(factory, created_pair)

  const exchange = new ExchangeContract(pair_address, client_a)
  await test_get_pair_info(exchange, created_pair, factory.address)

  await test_liquidity(exchange, sienna_token, created_pair)
  await sleep(SLEEP_TIME)

  await test_swap(exchange, factory, sienna_token, created_pair)

  await display_analytics()
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

  await execute_test_expect(
    'test_create_exchange_through_register_exchange_error',
    async () => { 
      const msg = {
        register_exchange: {
          pair,
          signature: 'whatever'
        }
      }

      const client_b = await build_client(ACC_B.mnemonic, APIURL)
      const client_c = await build_client(ACC_C.mnemonic, APIURL)

      const fee = create_fee('300000')

      const assert_unauthorized = (e: any) => {
        if (!e.message.includes('unauthorized')) {
          console.log(`"Error: register_exchange returned wrong error message: ${e.message}"`)
        }
      };

      const err_msg = 'Error: register_exchange should fail!'

      // Don't await these two in order to simulate multiple clients executing at once
      client_b.execute(factory.address, msg, undefined, undefined, fee)
        .then(
          () => console.log(err_msg),
          assert_unauthorized
        )
      
      client_c.execute(factory.address, msg, undefined, undefined, fee)
        .then(
          () => console.log(err_msg),
          assert_unauthorized
        )
      
      await factory.signing_client.execute(factory.address, msg, undefined, undefined, fee)
    },
    'unauthorized'
  )
  
  await execute_test(
    'test_create_exchange',
    async () => { 
      let result = await factory.create_exchange(pair)
      analytics.add_tx('Factory: Create Exchange', result)
    }
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
    'test_create_existing_pair_swapped_error',
    async () => { await factory.create_exchange(swapped) },
    'Pair already exists'
  )
}

async function test_query_exchanges(factory: FactoryContract, pair: TokenPair): Promise<Address> {
  let address = '';

  await execute_test(
    'test_get_exchange_address',
    async () => { 
      const result = await factory.get_exchange_address(pair)
      address = result
    }
  )

  await execute_test(
    'test_list_exchanges',
    async () => { 
      const result = await factory.list_exchanges(new Pagination(0, 30))
      assert_equal(result.length, 1)
      assert_equal(result[0].address, address)
      assert_objects_equal(result[0].pair, pair)
    }
  )

  return address
}

async function test_get_pair_info(exchange: ExchangeContract, pair: TokenPair, factory_address: Address) {
  await execute_test(
    'test_get_pair_info',
    async () => {
      const result = await exchange.get_pair_info()
      assert_objects_equal(pair, result.pair)
      assert_equal(result.amount_0, '0')
      assert_equal(result.amount_1, '0')
      assert_equal(result.total_liquidity, '0')
      assert_equal(factory_address, result.factory.address)
    }
  )
}

async function test_liquidity(exchange: ExchangeContract, sienna_token: ContractInfo, pair: TokenPair) {
  const amount = '5000000'

  // TODO: The current snip20 implementation doesn't implement
  // decimal conversion, so providing only a single amount for now
  //const amount1 = '5000000000000000000'

  const snip20 = new Snip20Contract(sienna_token.address, exchange.signing_client)
  await snip20_deposit(snip20, amount, exchange.address)

  const token_amount = new TokenPairAmount(pair, amount, amount)

  await execute_test(
    'test_provide_liquidity',
    async () => {
      const result = await exchange.provide_liquidity(token_amount)
      analytics.add_tx('Exchange: Provide Liquidity', result)

      assert_equal(extract_log_value(result, 'share'), amount) //LP tokens
    }
  )

  await execute_test(
    'test_provide_liquidity_pool_not_empty',
    async () => {
      const result = await exchange.get_pair_info()

      assert_equal(result.amount_0, amount)
      assert_equal(result.amount_1, amount)
    }
  )

  await sleep(SLEEP_TIME)

  await execute_test(
    'test_withdraw_liquidity',
    async () => {
      const result = await exchange.withdraw_liquidity(amount, exchange.signing_client.senderAddress)
      analytics.add_tx('Exchange: Withdraw Liquidity', result)

      assert_equal(extract_log_value(result, 'withdrawn_share'), amount)
      assert_equal(result.logs[0].events[1].attributes[0].value, exchange.signing_client.senderAddress)
    }
  )

  await sleep(SLEEP_TIME)

  await execute_test(
    'test_pool_empty_after_withdraw',
    async () => {
      const result = await exchange.get_pair_info()
      
      assert_equal(result.amount_0, '0')
      assert_equal(result.amount_1, '0')
    }
  )
}

async function test_swap(
  exchange: ExchangeContract,
  factory: FactoryContract,
  sienna_token: ContractInfo,
  pair: TokenPair
) {
  const amount = '5000000'

  // Setup liquidity pool
  const snip20_a = new Snip20Contract(sienna_token.address, exchange.signing_client)
  await snip20_deposit(snip20_a, amount, exchange.address)

  const pair_amount = new TokenPairAmount(pair, amount, amount)
  await exchange.provide_liquidity(pair_amount)

  await sleep(SLEEP_TIME)

  const client_b = await build_client(ACC_B.mnemonic, APIURL)
  const exchange_b = new ExchangeContract(exchange.address, client_b)
  const snip20_b = new Snip20Contract(sienna_token.address, client_b)
  
  const offer_token = new TokenTypeAmount(pair.token_0, '6000000') // swap uscrt for sienna

  await execute_test(
    'test_swap_simulation',
    async () => {
      exchange_b.simulate_swap(offer_token)

      const pool = await exchange_b.get_pair_info()
      
      assert_equal(pool.amount_0, amount)
      assert_equal(pool.amount_1, amount)
    }
  )

  await execute_test(
    'test_swap_from_native',
    async () => {
      const balance_before = parseInt(await get_native_balance(client_b));
      const result = await exchange_b.swap(offer_token)
      const balance_after = parseInt(await get_native_balance(client_b));

      analytics.add_tx('Exchange: Native Swap', result)
      
      assert(balance_before > balance_after) // TODO: calculate exact amount after adding gas parameters

      const pool = await exchange_b.get_pair_info()

      const amnt = parseInt(amount)
      const amount_0 = parseInt(pool.amount_0)
      const amount_1 = parseInt(pool.amount_1)

      assert(amnt < amount_0)
      assert(amnt > amount_1)

      const return_amount = parseInt(extract_log_value(result, 'return_amount') as string)
      assert(amnt - return_amount === amount_1)
    }
  )

  await snip20_deposit(snip20_b, amount, exchange.address)
  
  const key = create_viewing_key()
  await snip20_b.set_viewing_key(key)

  await execute_test(
    'test_get_allowance',
    async () => {
      const result = await snip20_b.get_allowance(client_b.senderAddress, exchange.address, key)
      assert_equal(result.allowance, amount)
    }
  )

  await execute_test_expect(
    'test_swap_from_snip20_insufficient_allowance',
    async () => {
      await exchange_b.swap(new TokenTypeAmount(pair.token_1, '99999999999999'))
    },
    'insufficient allowance:'
  )

  await execute_test(
    'test_swap_from_snip20',
    async () => {
      const native_balance_before = parseInt(await get_native_balance(client_b))
      const token_balance_before = parseInt(await snip20_b.get_balance(client_b.senderAddress, key))

      const swap_amount = '3000000'    

      const result = await exchange_b.swap(new TokenTypeAmount(pair.token_1, swap_amount))
      analytics.add_tx('Exchange: SNIP20 Swap', result)

      const native_balance_after = parseInt(await get_native_balance(client_b))
      const token_balance_after = parseInt(await snip20_b.get_balance(client_b.senderAddress, key))
      
      assert(native_balance_after > native_balance_before)
      assert(token_balance_before - parseInt(swap_amount) === token_balance_after)
      assert_equal(extract_log_value(result, 'sienna_commission'), '0')
    }
  )

  const client_burner = await build_client(BURN_POOL.mnemonic, APIURL)
  const snip20_burner = new Snip20Contract(sienna_token.address, client_burner)

  const key_burner = create_viewing_key()
  await snip20_burner.set_viewing_key(key_burner)

  await execute_test(
    'test_swap_with_burner',
    async () => {
      let config = get_exchange_settings();
      config.sienna_burner = BURN_POOL.address
      await factory.set_config(undefined, undefined, undefined, undefined, config)

      const token_balance_before = parseInt(await snip20_b.get_balance(client_b.senderAddress, key))

      const amount_to_swap = 3500000;
      const allowance = await snip20_b.get_allowance(client_b.senderAddress, exchange.address, key)

      await snip20_b.increase_allowance(
        exchange.address,
        (amount_to_swap - parseInt(allowance.allowance)).toString()
      )

      const result = await exchange_b.swap(new TokenTypeAmount(pair.token_1, amount_to_swap.toString()))
      analytics.add_tx('Exchange: Swap With Burner', result)

      const token_balance_after = parseInt(await snip20_b.get_balance(client_b.senderAddress, key))
      const burner_balance = parseInt(await snip20_burner.get_balance(client_burner.senderAddress, key_burner))

      const expected_commission = 700

      assert_equal(extract_log_value(result, 'sienna_commission'), expected_commission.toString())
      assert(burner_balance === expected_commission)
      assert(token_balance_before - amount_to_swap === token_balance_after)
    }
  )

  await execute_test(
    'test_swap_from_native_with_burner',
    async () => {
      const native_balance_before = parseInt(await get_native_balance(client_burner))
      const token_balance_before = parseInt(await snip20_b.get_balance(client_b.senderAddress, key))
      const burner_balance_before = parseInt(await snip20_burner.get_balance(client_burner.senderAddress, key_burner))

      const allowance = await snip20_b.get_allowance(client_b.senderAddress, exchange.address, key)

      const amount_to_swap = parseInt(offer_token.amount)

      await snip20_b.increase_allowance(
        exchange.address,
        (amount_to_swap - parseInt(allowance.allowance)).toString()
      )

      const result = await exchange_b.swap(offer_token)
      analytics.add_tx('Exchange: Native Swap With Burner', result)

      const token_balance_after = parseInt(await snip20_b.get_balance(client_b.senderAddress, key))
      const burner_balance_after = parseInt(await snip20_burner.get_balance(client_burner.senderAddress, key_burner))

      const native_balance_after = parseInt(await get_native_balance(client_burner))

      const expected_commission = 1200

      assert_equal(extract_log_value(result, 'sienna_commission'), expected_commission.toString())
      assert(burner_balance_after === burner_balance_before)
      assert(native_balance_after === native_balance_before + expected_commission)
      assert(token_balance_after > token_balance_before)
    }
  )
}

async function snip20_deposit(snip20: Snip20Contract, amount: Uint128, exchange_address: Address) {
  await snip20.deposit(amount)
  await sleep(SLEEP_TIME)

  await snip20.increase_allowance(exchange_address, amount)
  await sleep(SLEEP_TIME)
}

async function get_native_balance(client: SigningCosmWasmClient): Promise<string> {
  const account = await client.getAccount() as Account
  return account.balance[0].amount
}

function create_viewing_key(): ViewingKey {
  const rand_bytes = Random.getBytes(32)
  const key = new Sha256(rand_bytes).digest()

  return Buffer.from(key).toString('base64')
}

async function instantiate_factory(client: SigningCosmWasmClient, result: UploadResult): Promise<FactoryContract> {
  const factory_init_msg = {
    snip20_contract: result.snip20,
    lp_token_contract: result.lp_token,
    pair_contract: result.exchange,
    ido_contract: result.ido,
    exchange_settings: get_exchange_settings(),
    prng_seed: create_rand_base64()
  }

  const factory_instance = await client.instantiate(
    result.factory.id,
    factory_init_msg,
    'SIENNA AMM FACTORY',
    undefined,
    undefined,
    create_fee('200000')
  )

  return new FactoryContract(factory_instance.contractAddress, client)
}

async function instantiate_sienna_token(client: SigningCosmWasmClient, snip20: ContractInstantiationInfo): Promise<ContractInfo> {
  const init_msg = {
    name: 'sienna',
    symbol: 'SIENNA',
    decimals: 18,
    prng_seed: create_rand_base64(),
    config: {
      enable_burn: false,
      enable_deposit: true,
      enable_mint: true,
      enable_redeem: true,
      public_total_supply: true
    }
  }

  const sienna_contract = await client.instantiate(
    snip20.id,
    init_msg,
    'SIENNA TOKEN',
    undefined,
    undefined,
    create_fee('200000')
  )
  
  return new ContractInfo(snip20.code_hash, sienna_contract.contractAddress)
}

async function display_analytics() {
  const gas = await analytics.get_gas_usage()

  const gas_table = [ [ 'TX Name', 'Gas Wanted', 'Gas Used' ] ]
  gas.forEach(x => gas_table.push([ x.name, x.gas_wanted, x.gas_used ]))

  console.log(`\n Gas Usage:\n${table(gas_table)}`)
}

function get_exchange_settings(): ExchangeSettings {
  return {
    swap_fee: {
      nom: 28,
      denom: 1000
    },
    sienna_fee: {
        nom: 2,
        denom: 10000
    },
    sienna_burner: undefined
  }
}

run_tests().catch(console.log)

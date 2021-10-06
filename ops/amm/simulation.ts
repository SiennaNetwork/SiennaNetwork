import { AmmFactoryContract } from '../../api/siennajs/lib/amm_factory'
import { ExchangeContract } from '../../api/siennajs/lib/exchange'
import { Snip20Contract, TokenInfo } from '../../api/siennajs/lib/snip20'
import {
    Address, ContractInstantiationInfo, TokenPair, TokenPairAmount,
    TokenType, CustomToken, Uint128, TokenTypeAmount, create_coin,
    create_fee
} from '../../api/siennajs/lib/core'
import {
    upload_amm, build_client, create_rand_base64, create_account
} from './setup'
import { NullJsonFileWriter } from './utils/json_file_writer'
import { extract_log_value } from './utils/test_helpers'
import { APIURL, ACC, instantiate_factory } from './localnet'
import { TxAnalytics } from './utils/tx_analytics'

import { SigningCosmWasmClient, CosmWasmClient } from 'secretjs'
import { table } from 'table';

const NUM_USERS = 20
const NUM_PAIRS = 5
const NUM_RUNS = 50

const MIN_AMOUNT = 5
const MAX_AMOUNT = 300

const INITIAL_BALANCE = Math.max(1000000, NUM_RUNS * MAX_AMOUNT)
const INITIAL_LIQUIDITY = 5000

const USERS: User[] = []
const LIQUIDITY_PROVIDERS = new Map<Address, LiquidityProvider>()
const TOKENS = new Map<Address, TokenInfo>()
const PAIRS: PairContract[] = []

const LOG = [ [ 'Action', 'User', 'Pair', 'Description', 'Pool', 'Gas Wanted', 'Gas Used' ] ]
const QUERY_CLIENT = new CosmWasmClient(APIURL)

interface User {
    name: string,
    client: SigningCosmWasmClient
}

interface PairContract {
    name: string,
    address: Address,
    pair: TokenPair
}

interface LiquidityProvider {
    info: User,
    liquidity: Map<PairContract, bigint>
}

const analytics = new TxAnalytics(APIURL)

const rand = (low: number, high: number) => Math.round(Math.random() * (high - low) + low)

async function simulation() {
    const client = await build_client(ACC[0].mnemonic, APIURL)

    const result = await upload_amm(client, new NullJsonFileWriter)

    process.stdout.write(`Instantiating factory...\r`)
    const factory = await instantiate_factory(client, result, ACC[1].address)
    process.stdout.write(`Instantiating factory...done\r\n\n`)

    await create_users(client)
    await create_pairs(client, result.snip20, factory)
    await provide_initial_liquidity()

    await run_simulation()
}

async function create_users(client: SigningCosmWasmClient) {
    for(let i = 1; i <= NUM_USERS; i++) {
        process.stdout.write(`Creating user ${i} of ${NUM_USERS}\r`)
        const acc = await create_account()

        await client.sendTokens(
            acc.address,
            [ create_coin('100000000') ], // 100 SCRT
            undefined,
            create_fee('68000')
        )

        USERS.push({
            name: `User ${i}`,
            client: await build_client(acc.mnemonic, APIURL)
        })
    }

    console.log()
}

async function create_pairs(
    client: SigningCosmWasmClient,
    info: ContractInstantiationInfo,
    factory: AmmFactoryContract
) {
    const tokens: Address[][] = []

    const num_tokens = NUM_PAIRS * 2;
    let index = -1;

    for(let i = 1; i <= num_tokens; i++) {
        process.stdout.write(`Creating token ${i} of ${num_tokens}\r`)

        if (i % 2 === 1) {
            index++
            tokens[index] = []
        }

        const decimals = rand(6, 18)

        const initial_balances = []

        for(let i = 0; i < USERS.length; i++) {
            initial_balances.push({
                address: USERS[i].client.senderAddress,
                amount: raw_amount(INITIAL_BALANCE, decimals)
            })
        }

        const name = `Token ${i}`
        const symbol = 'TOKEN'

        const msg = {
            name,
            symbol,
            decimals,
            prng_seed: create_rand_base64(),
            initial_balances,
            config: {
                enable_burn: false,
                enable_deposit: true,
                enable_mint: true,
                enable_redeem: true,
                public_total_supply: true
            }
        }

        const result = await client.instantiate(info.id, msg, name, undefined, undefined, create_fee('2700000'))
        tokens[index].push(result.contractAddress)

        TOKENS.set(result.contractAddress, {
            name,
            symbol,
            decimals,
            total_supply: '0'
        })
    }

    console.log()

    for(let i = 1; i <= NUM_PAIRS; i++) {
        process.stdout.write(`Creating pair ${i} of ${NUM_PAIRS}\r`)

        const addresses = tokens[i - 1]
        const pair = new TokenPair({
                custom_token: {
                    contract_addr: addresses[0],
                    token_code_hash: info.code_hash
                }
            },{
                custom_token: {
                    contract_addr: addresses[1],
                    token_code_hash: info.code_hash
                }
            }
        )

        const result = await factory.exec().create_exchange(pair)

        const info_0 = TOKENS.get(addresses[0]) as TokenInfo
        const info_1 = TOKENS.get(addresses[1]) as TokenInfo

        PAIRS.push({
            name: `${info_0.name} <==> ${info_1.name}`,
            address: extract_log_value(result, 'created_exchange_address') as string,
            pair
        })
    }

    console.log()
}

async function provide_initial_liquidity() {
    for(let i = 0; i < PAIRS.length; i++) {
        const user_index = rand(0, USERS.length - 1)
        await provide_liquidity(USERS[user_index], PAIRS[i], INITIAL_LIQUIDITY)
    }
}

async function run_simulation() {
    for(let i = 0; i < NUM_RUNS; i++) {
        const roll = rand(1, 100)

        if (roll <= 10) {
            await withdraw_liquidity()
        } else if (roll <= 40) {
            const user = USERS[rand(0, USERS.length - 1)]
            const pair = PAIRS[rand(0, PAIRS.length - 1)]
            const amount = rand(MIN_AMOUNT, MAX_AMOUNT)

            await provide_liquidity(user, pair, amount)
        } else {
            await swap()
        }
    }
}

async function provide_liquidity(user: User, pair: PairContract, amount: number) {
    const contract = new ExchangeContract(pair.address, user.client)

    const snip20_contract_0 = new Snip20Contract((pair.pair.token_0 as CustomToken).custom_token.contract_addr, user.client)
    const snip20_contract_1 = new Snip20Contract((pair.pair.token_1 as CustomToken).custom_token.contract_addr, user.client)

    const amount_0 = raw_amount(amount, get_decimals(pair.pair.token_0))
    const amount_1 = raw_amount(amount, get_decimals(pair.pair.token_1))

    await snip20_contract_0.exec().increase_allowance(pair.address, amount_0)
    await snip20_contract_1.exec().increase_allowance(pair.address, amount_1)

    const result = await contract.exec().provide_liquidity(new TokenPairAmount(
        pair.pair,
        amount_0,
        amount_1
    ))

    const lp_amount = BigInt(extract_log_value(result, 'share') as string)

    let lp = LIQUIDITY_PROVIDERS.get(user.client.senderAddress)

    if (lp === undefined) {
        lp = {
            info: user,
            liquidity: new Map<PairContract, bigint>()
        }

        lp.liquidity.set(pair, BigInt(lp_amount))

        LIQUIDITY_PROVIDERS.set(user.client.senderAddress, lp)
    } else {
        let liquidity = lp.liquidity.get(pair)

        if (liquidity === undefined) {
            lp.liquidity.set(pair, lp_amount)
        } else {
            lp.liquidity.set(pair, liquidity + lp_amount)
        }
    }

    await log_action(
        'Provide',
        user.name,
        `Amounts: ${amount_0}, ${amount_1}`,
        result.transactionHash,
        pair
    )
}

async function withdraw_liquidity() {
    const lps = Array.from(LIQUIDITY_PROVIDERS.keys())

    const lp_index = rand(0, lps.length - 1)
    const lp_address = lps[lp_index]

    const lp = LIQUIDITY_PROVIDERS.get(lp_address) as LiquidityProvider
    
    const pairs = Array.from(lp.liquidity.keys())
    const pair_index = rand(0, pairs.length - 1)
    const pair = pairs[pair_index]

    const lp_amount = lp.liquidity.get(pair) as bigint
    let rand_amount = BigInt(raw_amount(rand(MIN_AMOUNT, MAX_AMOUNT), 18)) // LP tokens has 18 decimals
    
    if (rand_amount >= lp_amount) {
        rand_amount = lp_amount
        lp.liquidity.delete(pair)

        if (lp.liquidity.size === 0) {
            LIQUIDITY_PROVIDERS.delete(lp_address)
        }
    } else {
        lp.liquidity.set(pair, lp_amount - rand_amount)
    }

    const contract = new ExchangeContract(pair.address, lp.info.client)
    const result = await contract.exec().withdraw_liquidity(rand_amount.toString(), lp.info.client.senderAddress)

    await log_action(
        'Withdraw',
        lp.info.name,
        `Amount: ${rand_amount}`,
        result.transactionHash,
        pair
    )
}

async function swap() {
    const user = USERS[rand(0, USERS.length - 1)]
    const pair = PAIRS[rand(0, PAIRS.length - 1)]

    if (rand(0, 1) === 0) {
        var token = pair.pair.token_0
    } else {
        var token = pair.pair.token_1
    }

    const info = get_token_info(token)
    const amount = raw_amount(rand(MIN_AMOUNT, MAX_AMOUNT), info.decimals)

    const contract = new ExchangeContract(pair.address, user.client)
    const result = await contract.exec().swap(new TokenTypeAmount(token, amount))

    await log_action(
        'Swap',
        user.name,
        `Amount: ${amount}, Token: ${info.name}`,
        result.transactionHash,
        pair
    )
}

function get_decimals(token: TokenType): number {
    const info = get_token_info(token)
    return info.decimals
}

function get_token_info(token: TokenType): TokenInfo {
    const addr = (token as CustomToken).custom_token.contract_addr
    return TOKENS.get(addr) as TokenInfo
}

function raw_amount(amount: number, decimals: number): Uint128 {
    return BigInt((amount * 10 ** decimals)).toString()
}

async function log_action(
    action: string,
    user_name: string,
    desc: string,
    hash: string,
    pair: PairContract
) {
    const contract = new ExchangeContract(pair.address, undefined, QUERY_CLIENT)
    
    const pool = await contract.query().get_pair_info()
    const name_0 = get_token_info(pool.pair.token_0).name
    const name_1 = get_token_info(pool.pair.token_1).name

    const amounts = `${name_0} (${pool.amount_0}) / ${name_1} (${pool.amount_1})`

    const gas = await analytics.get_gas_usage(hash)
    LOG.push([ action, user_name, pair.name, desc, amounts, gas.gas_wanted, gas.gas_used ])

    console.log(`${table(LOG)}`)
}

simulation().catch(console.log)

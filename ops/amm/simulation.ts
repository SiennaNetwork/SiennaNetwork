import { FactoryContract, create_coin, create_fee } from './amm-lib/contract.js'
import { Address, ContractInfo, ContractInstantiationInfo } from './amm-lib/types.js'
import {
    upload_amm, build_client, create_rand_base64, create_account
} from './setup.js'
import { NullJsonFileWriter } from './utils/json_file_writer.js'
import { APIURL, ACC, instantiate_factory } from './localnet.js'

import { SigningCosmWasmClient } from 'secretjs'

const NUM_USERS: number = 5
const USERS: Address[] = []

const NUM_PAIRS: number = 3
const TOKENS: ContractInfo[][] = []

const rand = (low: number, high: number) => Math.round(Math.random() * (high - low) + low)

async function simulation() {
    const client = await build_client(ACC[0].mnemonic, APIURL)

    process.stdout.write(`Uploading contracts...\r`)
    const result = await upload_amm(client, new NullJsonFileWriter)
    process.stdout.write(`Uploading contracts...done\r\n`)

    process.stdout.write(`Instantiating factory...\r`)
    const factory = await instantiate_factory(client, result)
    process.stdout.write(`Instantiating factory...done\r\n`)

    await create_users(client)
    await create_pairs(client, result.snip20)
}

async function create_users(client: SigningCosmWasmClient) {
    for(let i = 1; i <= NUM_USERS; i++) {
        process.stdout.write(`Creating user ${i} of ${NUM_USERS}\r`)

        const acc = await create_account()
        await client.sendTokens(acc, [ create_coin('100000000') ], undefined, create_fee('100000')) // 100 SCRT

        USERS.push(acc)
    }

    console.log()
}

async function create_pairs(
    client: SigningCosmWasmClient,
    info: ContractInstantiationInfo
) {
    const num_tokens = NUM_PAIRS * 2;
    let index = -1;

    for(let i = 1; i <= num_tokens; i++) {
        process.stdout.write(`Creating token ${i} of ${num_tokens}\r`)

        if (i % 2 === 1) {
            index++
            TOKENS[index] = []
        }

        const name = `Token ${i}`
        const msg = {
            name,
            symbol: `TOKEN`,
            decimals: rand(6, 18),
            prng_seed: create_rand_base64(),
            config: {
                enable_burn: false,
                enable_deposit: true,
                enable_mint: true,
                enable_redeem: true,
                public_total_supply: true
            }
        }

        const result = await client.instantiate(info.id, msg, name, undefined, undefined, create_fee('200000'))
        TOKENS[index].push(new ContractInfo(info.code_hash, result.contractAddress))
    }

    console.log()
}

simulation().catch(console.log)

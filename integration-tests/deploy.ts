import { ContractInfo } from './amm-lib/types.js'
import { setup, build_client } from './setup.js'
import { JsonFileWriter } from './utils/json_file_writer.js'
import { Random } from "@iov/crypto"
import { create_fee, Snip20Contract } from './amm-lib/contract.js'
import { resolve } from 'path'
import { readFileSync } from 'fs'
import * as dotenv from 'dotenv'

import { createInterface } from 'readline'

dotenv.config()

async function deploy() {
    const client = await build_client(process.env.MNEMONIC as string, process.env.SECRET_REST_URL as string)

    let address = ''
    let code_hash = ''

    let readline = createInterface({
        input: process.stdin,
        output: process.stdout
    })

    readline.question('SIENNA token address?', (addr) => {
        address = addr

        readline.question("SIENNA token code hash (corresponds to the 'originalChecksum' field)?", async (hash) => {
            code_hash = hash
            readline.close()

            const info = new ContractInfo(code_hash, address)
            const writer = new JsonFileWriter(`../dist/${process.env.SECRET_CHAIN_ID}/`)

            await setup(client, process.argv[2], info, writer)
        })
    })
}

async function snip20() {
    /*
    const fee = create_fee('2000000')

    const client = await build_client(process.env.MNEMONIC as string, process.env.SECRET_REST_URL as string)
    const writer = new JsonFileWriter(`../dist/${process.env.SECRET_CHAIN_ID}/`)

    const msg = {
        name: 'test_token2',
        symbol: 'SITOK',
        decimals: 6,
        prng_seed: create_rand_base64()
    } 

    const token_contract = await client.instantiate(28558, msg, 'SIENNA_TEST2', undefined, undefined, fee)
    const info = new ContractInfo('78cb50a550d579eb671e05e868d26ba48f5201a2d23250c635269c889c7db829', token_contract.contractAddress)

    const obj = {
        contract: info,
        info: msg
    }

    writer.write(obj, `test-tokens/test_token_2`)
    */
    const client = await build_client(process.env.MNEMONIC as string, process.env.SECRET_REST_URL as string)
    const snip20 = new Snip20Contract('secret129nq840d05a0tvkranw5xesq9k0uwmn8mg7ft5', client)
    
    const amount = '5000000000000'

    await snip20.mint('secret1dej4qmvfxcdvmea4ku9fvzvk8zdu3rvyujhrt6', amount)
}

function create_rand_base64(): string {
    const rand_bytes = Random.getBytes(32)
    return Buffer.from(rand_bytes).toString('base64')
}

snip20().catch(console.log)
//deploy().catch(console.log)

import { ContractInfo } from './amm-lib/types.js'
import { setup, build_client } from './setup.js'
import { JsonFileWriter } from './utils/json_file_writer.js'

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
            const writer = new JsonFileWriter('../dist/')

            await setup(client, process.argv[2], info, writer)
        })
    })
}

deploy().catch(console.log)

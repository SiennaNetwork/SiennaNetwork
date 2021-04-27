import { setup, build_client } from './setup.js'
import { JsonFileWriter } from './utils/json_file_writer.js'
import * as dotenv from 'dotenv'

dotenv.config()

async function deploy() {
    const client = await build_client(process.env.MNEMONIC as string, process.env.SECRET_REST_URL as string)
    const writer = new JsonFileWriter(`../dist/${process.env.SECRET_CHAIN_ID}/`)

    await setup(client, process.argv[2], writer)
}

deploy().catch(console.log)

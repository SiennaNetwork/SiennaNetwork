import { build_client, read_config } from './setup.js'
import { FactoryContract } from './amm-lib/amm_factory.js'
import * as dotenv from 'dotenv'

dotenv.config()

async function change_config() {
    const address = process.argv[2]

    if(!address) {
        console.log("Please provide a factory address.")

        return
    }

    const config = read_config(process.env.SECRET_CHAIN_ID as string, (file) => {
        console.log(`Couldn't find config file "${file}" for chain "${process.env.SECRET_CHAIN_ID}".`)
    })

    const client = await build_client(process.env.MNEMONIC as string, process.env.SECRET_REST_URL as string)
    const factory = new FactoryContract(address, client)

    await factory.set_config(undefined, undefined, undefined, undefined, config.exchange_settings)
}

change_config().catch(console.log)

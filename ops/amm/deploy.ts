import { upload, build_client, read_config } from './setup.js'
import { JsonFileWriter } from './utils/json_file_writer.js'
import * as dotenv from 'dotenv'
import { ContractInfo } from './amm-lib/types.js'
import { create_fee } from './amm-lib/contract.js'
import { writeFileSync } from 'fs'

dotenv.config()

async function deploy() {
    let config = {
        exchange_settings: {
            swap_fee: {
                nom: 28,
                denom: 1000
            },
            sienna_fee: {
                nom: 2,
                denom: 10000
            },
            sienna_burner: null
        },
        admin: null
    }

    config = read_config(process.env.SECRET_CHAIN_ID as string, (file) => {
        writeFileSync(file, JSON.stringify(config, null, 2))
        console.log(`Couldn't find file "${file}". Created one with default values. Please configure it and run this script again.`)
    })

    const client = await build_client(process.env.MNEMONIC as string, process.env.SECRET_REST_URL as string)
    const writer = new JsonFileWriter(`../../artifacts/amm/${process.env.SECRET_CHAIN_ID}/`)

    const result = await upload(client, writer)

    const factory_init_msg = {
        snip20_contract: result.snip20,
        lp_token_contract: result.lp_token,
        pair_contract: result.exchange,
        ido_contract: result.ido
    }

    Object.assign(factory_init_msg, config)

    const commit = process.argv[2];
    const factory_instance = await client.instantiate(
        result.factory.id,
        factory_init_msg,
        `${commit} - SIENNA AMM FACTORY`,
        undefined,
        undefined,
        create_fee('200000')
    )

    writer.write(
        new ContractInfo(result.factory.code_hash, factory_instance.contractAddress),
        `addresses/factory`
    )
}

deploy().catch(console.log)

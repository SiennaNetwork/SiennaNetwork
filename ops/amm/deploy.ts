import { upload, build_client } from './setup.js'
import { JsonFileWriter } from './utils/json_file_writer.js'
import * as dotenv from 'dotenv'
import { ContractInfo } from './amm-lib/types.js'
import { create_fee } from './amm-lib/contract.js'

dotenv.config()

async function deploy() {
    const client = await build_client(process.env.MNEMONIC as string, process.env.SECRET_REST_URL as string)
    const writer = new JsonFileWriter(`../../artifacts/amm/${process.env.SECRET_CHAIN_ID}/`)

    const result = await upload(client, writer)

    // TODO: Pull from config file
    const factory_init_msg = {
        snip20_contract: result.snip20,
        lp_token_contract: result.lp_token,
        pair_contract: result.exchange,
        ido_contract: result.ido,
        exchange_settings: {
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

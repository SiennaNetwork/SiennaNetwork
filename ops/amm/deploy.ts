import { upload_amm, build_client, read_config, ARTIFACTS_PATH, create_rand_base64 } from './setup.js'
import { JsonFileWriter } from './utils/json_file_writer.js'
import * as dotenv from 'dotenv'
import { ContractInfo } from './amm-lib/types.js'
import { create_fee } from './amm-lib/contract.js'

import { writeFileSync, readFileSync } from 'fs'
import { resolve } from 'path'

dotenv.config()

const configs = [
    {
        amm: {
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
    },
    {
        rewards: [
            {
                admin: null,
                reward_token: {
                    address: '',
                    code_hash: ''
                },
                lp_token: {
                    address: '',
                    code_hash: ''
                },
                ratio: [ 1, 1 ],
                threshold: 17280 // ~24h @ 5s/block
            }
        ]
    }
]

async function deploy() {
    const options = configs.flatMap(x => Object.keys(x))
    const selected = process.argv[3]
    
    if (!options.includes(selected)) {
        console.log(`Expecting argument. One of: [ ${options.join(', ')} ]`)
        return
    }
    
    let config = configs.find(x => Object.keys(x)[0] == selected) as any
    config = Object.values(config)[0]

    const file_name = `${selected}-${process.env.SECRET_CHAIN_ID}`
    config = read_config(file_name, (file) => {
        writeFileSync(file, JSON.stringify(config, null, 2))
        console.log(`Couldn't find file "${file}". Created one with default values. Please configure it and run this script again.`)

        process.exit(0)
    })

    switch (selected) {
        case 'amm':
            await deploy_amm(config)
            break
        case 'rewards':
            await deploy_rewards(config)
            break
        default:
            console.log(`No action for option "${selected}" found.`)
            break
    }
}

async function deploy_amm(config: any) {
    const client = await build_client(process.env.MNEMONIC as string, process.env.SECRET_REST_URL as string)
    const writer = new JsonFileWriter(`${ARTIFACTS_PATH}/amm/${process.env.SECRET_CHAIN_ID}/`)

    const result = await upload_amm(client, writer)

    const init_msg = {
        snip20_contract: result.snip20,
        lp_token_contract: result.lp_token,
        pair_contract: result.exchange,
        ido_contract: result.ido,
        prng_seed: create_rand_base64()
    }

    Object.assign(init_msg, config)

    const commit = process.argv[2];
    const instance = await client.instantiate(
        result.factory.id,
        init_msg,
        `SIENNA AMM FACTORY (${commit})`,
        undefined,
        undefined,
        create_fee('200000')
    )

    writer.write(
        new ContractInfo(result.factory.code_hash, instance.contractAddress),
        `addresses/factory`
    )
}

async function deploy_rewards(config: any[]) {
    const client = await build_client(process.env.MNEMONIC as string, process.env.SECRET_REST_URL as string)
    const writer = new JsonFileWriter(`${ARTIFACTS_PATH}/rewards/${process.env.SECRET_CHAIN_ID}/`)

    const wasm = readFileSync(resolve(`${ARTIFACTS_PATH}/sienna-rewards-benchmark@HEAD.wasm`))

    const upload = await client.upload(wasm, undefined, undefined, create_fee('1800000'))
    writer.write(upload, `uploads/rewards`)

    for(let obj of config) {
        const init_msg = {
            viewing_key: create_rand_base64()
        }
    
        Object.assign(init_msg, obj)

        const commit = process.argv[2];
        const lp_token_addr = obj.lp_token.address;

        const instance = await client.instantiate(
            upload.codeId,
            init_msg,
            `SIENNA REWARDS - ${lp_token_addr} (${commit})`,
            undefined,
            undefined,
            create_fee('270000')
        )
    
        writer.write(
            new ContractInfo(upload.originalChecksum, instance.contractAddress),
            `addresses/rewards-${lp_token_addr}`
        )
    }
}

deploy().catch(console.log)

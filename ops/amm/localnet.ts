import { FactoryContract, create_fee } from './amm-lib/contract.js'
import { ExchangeSettings } from './amm-lib/types.js'
import { create_rand_base64, UploadResult } from './setup.js'
import { SigningCosmWasmClient } from 'secretjs'

export interface LocalAccount {
    name: string,
    type: string,
    address: string,
    pubkey: string,
    mnemonic: string
}

export const APIURL = 'http://localhost:1337'
export const ACC: LocalAccount[] = JSON.parse(process.argv[2])

export async function instantiate_factory(client: SigningCosmWasmClient, result: UploadResult): Promise<FactoryContract> {
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

export function get_exchange_settings(): ExchangeSettings {
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

import { create_fee, Address } from '../../api/siennajs/lib/core'
import { ExchangeSettings } from '../../api/siennajs/lib/amm_factory'
import { create_rand_base64, UploadResult } from './setup'

import { SigningCosmWasmClient, InstantiateResult } from 'secretjs'

export interface LocalAccount {
    name: string,
    type: string,
    address: Address,
    pubkey: string,
    mnemonic: string
}

export const APIURL = 'http://localhost:1337'
export const ACC: LocalAccount[] = JSON.parse(process.argv[2])

export async function instantiate_factory(
    client: SigningCosmWasmClient,
    result: UploadResult,
    burner?: Address | undefined
): Promise<InstantiateResult> {
    const factory_init_msg = {
        snip20_contract: result.snip20,
        lp_token_contract: result.lp_token,
        pair_contract: result.exchange,
        ido_contract: result.ido,
        launchpad_contract: result.launchpad,
        exchange_settings: get_exchange_settings(burner),
        prng_seed: create_rand_base64()
    }
  
    const instance = await client.instantiate(
        result.factory.id,
        factory_init_msg,
        `SIENNA AMM FACTORY_${Date.now()}`,
        undefined,
        undefined,
        create_fee('165000')
    )
  
    return instance
}

export function get_exchange_settings(sienna_burner?: Address | undefined): ExchangeSettings {
    return {
        swap_fee: {
            nom: 28,
            denom: 1000
        },
        sienna_fee: {
            nom: 2,
            denom: 10000
        },
        sienna_burner
    }
}

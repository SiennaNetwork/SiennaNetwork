import { create_fee, Address } from '../../frontends/siennajs/lib/core'
import { ExchangeSettings } from '../../frontends/siennajs/lib/amm_factory'
import { create_rand_base64, UploadResult } from './setup'

import { SigningCosmWasmClient, InstantiateResult } from 'secretjs'

export interface LocalAccount {
    address: Address,
    mnemonic: string
}

export const APIURL = 'http://localhost:1337'
export const ACC: LocalAccount[] = parse_local_acc(process.argv[2])

function parse_local_acc(text: string): LocalAccount[] {
    const accounts = text.split('**END ACC**') // This phrase is in bootstrap_init.sh
    const result: LocalAccount[] = []

    const address_search = 'address: '
    const mnemonic_search = 'password.\n\n'

    for(let acc of accounts) {
        const address_start = acc.indexOf(address_search) + address_search.length
        const address_end = acc.indexOf(' pubkey:')
        const address = acc.substring(address_start, address_end).trimEnd()

        const mnemonic_start = acc.indexOf(mnemonic_search) + mnemonic_search.length
        const mnemonic = acc.substring(mnemonic_start).trimEnd()

        result.push({
            address,
            mnemonic
        })
    }
    
    return result
}

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

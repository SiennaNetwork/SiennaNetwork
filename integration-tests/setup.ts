import { ContractInfo, ContractInstantiationInfo } from './amm-lib/types.js'
import { FactoryContract, create_fee } from './amm-lib/contract.js'
import { IJsonFileWriter } from './utils/json_file_writer.js'

import { 
    SigningCosmWasmClient, Secp256k1Pen, encodeSecp256k1Pubkey,
    pubkeyToAddress, EnigmaUtils
} from 'secretjs'

import { resolve } from 'path'
import { readFileSync } from 'fs'

interface SetupResult {
    factory: FactoryContract,
    snip20_contract: ContractInstantiationInfo,
}

export async function setup(client: SigningCosmWasmClient, commit: string, writer: IJsonFileWriter): Promise<SetupResult> {
    const fee = create_fee('2000000')
  
    const snip20_wasm = readFileSync(resolve(`../dist/${commit}-snip20-reference-impl.wasm`))
    const exchange_wasm = readFileSync(resolve(`../dist/${commit}-exchange.wasm`))
    const factory_wasm = readFileSync(resolve(`../dist/${commit}-factory.wasm`))
    const lp_token_wasm = readFileSync(resolve(`../dist/${commit}-lp-token.wasm`))
    const ido_wasm = readFileSync(resolve(`../dist/ido.wasm`))

    const exchange_upload = await client.upload(exchange_wasm, { }, undefined, fee)
    writer.write(exchange_upload, `uploads/exchange`)

    const snip20_upload = await client.upload(snip20_wasm, {}, undefined, fee)
    writer.write(snip20_upload, `uploads/snip20`)

    const factory_upload = await client.upload(factory_wasm, {}, undefined, fee)
    writer.write(factory_upload, `uploads/factory`)

    const lp_token_upload = await client.upload(lp_token_wasm, {}, undefined, fee)
    writer.write(lp_token_upload, `uploads/lp_token`)

    const ido_upload = await client.upload(ido_wasm, {}, undefined, fee)
    writer.write(ido_upload, `uploads/ido`)

    //const burner_upload = await client.upload(ido_wasm, {}, undefined, fee)
    //writer.write(burner_upload, `uploads/burner`)

    const pair_contract = new ContractInstantiationInfo(exchange_upload.originalChecksum, exchange_upload.codeId)
    const snip20_contract = new ContractInstantiationInfo(snip20_upload.originalChecksum, snip20_upload.codeId)
    const lp_token_contract = new ContractInstantiationInfo(lp_token_upload.originalChecksum, lp_token_upload.codeId)
    const ido_contract = new ContractInstantiationInfo(ido_upload.originalChecksum, ido_upload.codeId)
  
    const factory_init_msg = {
        snip20_contract,
        lp_token_contract,
        pair_contract,
        ido_contract,
        exchange_settings: {
            swap_fee: {
                nom: 28,
                denom: 10000
            },
            sienna_fee: {
                nom: 2,
                denom: 10000
            },
            sienna_burner: undefined
        }
    }
    
    const result = await client.instantiate(factory_upload.codeId, factory_init_msg, `${commit} - AMM FACTORY`, undefined, undefined, fee)
    writer.write(
        new ContractInfo(factory_upload.originalChecksum, result.contractAddress),
        `addresses/factory`
    )

    const factory = new FactoryContract(result.contractAddress, client)
  
    return { factory, snip20_contract }
}

export async function build_client(mnemonic: string, api_url: string): Promise<SigningCosmWasmClient> {
    const pen = await Secp256k1Pen.fromMnemonic(mnemonic)
    const seed = EnigmaUtils.GenerateNewSeed();
  
    const pubkey  = encodeSecp256k1Pubkey(pen.pubkey)
    const address = pubkeyToAddress(pubkey, 'secret')
  
    return new SigningCosmWasmClient(
        api_url,
        address,
        (bytes) => pen.sign(bytes),
        seed
    )
}

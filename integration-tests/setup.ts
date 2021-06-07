import { ContractInstantiationInfo } from './amm-lib/types.js'
import { create_fee } from './amm-lib/contract.js'
import { IJsonFileWriter } from './utils/json_file_writer.js'

import { 
    SigningCosmWasmClient, Secp256k1Pen, encodeSecp256k1Pubkey,
    pubkeyToAddress, EnigmaUtils
} from 'secretjs'

import { resolve } from 'path'
import { readFileSync } from 'fs'

export interface UploadResult {
    factory: ContractInstantiationInfo,
    snip20: ContractInstantiationInfo,
    exchange: ContractInstantiationInfo,
    lp_token: ContractInstantiationInfo,
    ido: ContractInstantiationInfo
}

export async function upload(client: SigningCosmWasmClient, commit: string, writer: IJsonFileWriter): Promise<UploadResult> {
    const fee = create_fee('2500000')
  
    const snip20_wasm = readFileSync(resolve(`../dist/${commit}-snip20-reference-impl.wasm`))
    const exchange_wasm = readFileSync(resolve(`../dist/${commit}-exchange.wasm`))
    const factory_wasm = readFileSync(resolve(`../dist/${commit}-factory.wasm`))
    const lp_token_wasm = readFileSync(resolve(`../dist/${commit}-lp-token.wasm`))
    const ido_wasm = readFileSync(resolve(`../dist/${commit}-ido.wasm`))

    const exchange_upload = await client.upload(exchange_wasm, {}, undefined, fee)
    writer.write(exchange_upload, `uploads/exchange`)

    const snip20_upload = await client.upload(snip20_wasm, {}, undefined, fee)
    writer.write(snip20_upload, `uploads/snip20`)

    const factory_upload = await client.upload(factory_wasm, {}, undefined, fee)
    writer.write(factory_upload, `uploads/factory`)

    const lp_token_upload = await client.upload(lp_token_wasm, {}, undefined, fee)
    writer.write(lp_token_upload, `uploads/lp_token`)

    const ido_upload = await client.upload(ido_wasm, {}, undefined, fee)
    writer.write(ido_upload, `uploads/ido`)
  
    return { 
        factory: new ContractInstantiationInfo(factory_upload.originalChecksum, factory_upload.codeId),
        snip20: new ContractInstantiationInfo(snip20_upload.originalChecksum, snip20_upload.codeId),
        exchange: new ContractInstantiationInfo(exchange_upload.originalChecksum, exchange_upload.codeId),
        lp_token: new ContractInstantiationInfo(lp_token_upload.originalChecksum, lp_token_upload.codeId),
        ido: new ContractInstantiationInfo(ido_upload.originalChecksum, ido_upload.codeId)
    }
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

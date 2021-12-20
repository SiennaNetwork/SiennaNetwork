import { ContractInstantiationInfo, Address, create_fee } from '../../frontends/siennajs/lib/core'
import { IJsonFileWriter } from './utils/json_file_writer'
import { TxAnalytics } from './utils/tx_analytics'

import { 
    SigningCosmWasmClient, Secp256k1Pen, encodeSecp256k1Pubkey,
    pubkeyToAddress, EnigmaUtils
} from 'secretjs'
import { table } from 'table';

import { Random, Bip39 } from "@iov/crypto"

import { resolve } from 'path'
import { readFileSync } from 'fs'

export interface UploadResult {
    factory: ContractInstantiationInfo,
    snip20: ContractInstantiationInfo,
    exchange: ContractInstantiationInfo,
    lp_token: ContractInstantiationInfo,
    ido: ContractInstantiationInfo,
    launchpad: ContractInstantiationInfo,
    router: ContractInstantiationInfo
}

export interface ScrtAccount {
    address: Address,
    mnemonic: string
}

export const ARTIFACTS_PATH = '../../artifacts'

export async function upload_amm(client: SigningCosmWasmClient, writer: IJsonFileWriter): Promise<UploadResult> {
    const snip20_fee = create_fee('3200000')

    const url = (client as any).restClient.enigmautils.apiUrl;
    const analytics = new TxAnalytics(url)

    const snip20_wasm = readFileSync(resolve(`${ARTIFACTS_PATH}/amm-snip20@HEAD.wasm`))
    const exchange_wasm = readFileSync(resolve(`${ARTIFACTS_PATH}/exchange@HEAD.wasm`))
    const factory_wasm = readFileSync(resolve(`${ARTIFACTS_PATH}/factory@HEAD.wasm`))
    const lp_token_wasm = readFileSync(resolve(`${ARTIFACTS_PATH}/lp-token@HEAD.wasm`))
    const ido_wasm = readFileSync(resolve(`${ARTIFACTS_PATH}/ido@HEAD.wasm`))
    const launchpad_wasm = readFileSync(resolve(`${ARTIFACTS_PATH}/launchpad@HEAD.wasm`))
    const router_wasm = readFileSync(resolve(`${ARTIFACTS_PATH}/router@HEAD.wasm`))

    process.stdout.write(`Uploading exchange contract...\r`)
    const exchange_upload = await client.upload(exchange_wasm, {}, undefined, create_fee('2500000'))
    analytics.add_tx(exchange_upload.transactionHash, 'Exchange')
    writer.write(exchange_upload, `uploads/exchange`)
    process.stdout.write(`Uploading exchange contract...done\r\n`)

    process.stdout.write(`Uploading SNIP20 contract...\r`)
    const snip20_upload = await client.upload(snip20_wasm, {}, undefined, snip20_fee)
    analytics.add_tx(snip20_upload.transactionHash, 'SNIP20')
    writer.write(snip20_upload, `uploads/snip20`)
    process.stdout.write(`Uploading SNIP20 contract...done\r\n`)

    process.stdout.write(`Uploading factory contract...\r`)
    const factory_upload = await client.upload(factory_wasm, {}, undefined, create_fee('2300000'))
    analytics.add_tx(factory_upload.transactionHash, 'Factory')
    writer.write(factory_upload, `uploads/factory`)
    process.stdout.write(`Uploading factory contract...done\r\n`)

    process.stdout.write(`Uploading LP token contract...\r`)
    const lp_token_upload = await client.upload(lp_token_wasm, {}, undefined, snip20_fee)
    analytics.add_tx(lp_token_upload.transactionHash, 'LP Token')
    writer.write(lp_token_upload, `uploads/lp_token`)
    process.stdout.write(`Uploading LP token contract...done\r\n`)

    process.stdout.write(`Uploading IDO contract...\r`)
    const ido_upload = await client.upload(ido_wasm, {}, undefined, create_fee('2600000'))
    analytics.add_tx(ido_upload.transactionHash, 'IDO')
    writer.write(ido_upload, `uploads/ido`)
    process.stdout.write(`Uploading IDO contract...done\r\n`)

    process.stdout.write(`Uploading launchpad contract...\r`)
    const launchpad_upload = await client.upload(launchpad_wasm, {}, undefined, create_fee('2500000'))
    analytics.add_tx(launchpad_upload.transactionHash, 'Launchpad')
    writer.write(launchpad_upload, `uploads/launchpad`)
    process.stdout.write(`Uploading launchpad contract...done\r\n`)

    process.stdout.write(`Uploading router contract...\r`)
    const router_upload = await client.upload(router_wasm, {}, undefined, create_fee('1600000'))
    analytics.add_tx(router_upload.transactionHash, 'Router')
    writer.write(router_upload, `uploads/router`)
    process.stdout.write(`Uploading router contract...done\r\n`)

    const gas = await analytics.get_gas_report()
    const gas_table = [ [ 'Uploaded Contract', 'Gas Wanted', 'Gas Used' ] ]

    gas.forEach(x => gas_table.push([ x.name, x.gas_wanted, x.gas_used ]))
    console.log(table(gas_table))
  
    return { 
        factory: new ContractInstantiationInfo(factory_upload.originalChecksum, factory_upload.codeId),
        snip20: new ContractInstantiationInfo(snip20_upload.originalChecksum, snip20_upload.codeId),
        exchange: new ContractInstantiationInfo(exchange_upload.originalChecksum, exchange_upload.codeId),
        lp_token: new ContractInstantiationInfo(lp_token_upload.originalChecksum, lp_token_upload.codeId),
        ido: new ContractInstantiationInfo(ido_upload.originalChecksum, ido_upload.codeId),
        launchpad: new ContractInstantiationInfo(launchpad_upload.originalChecksum, launchpad_upload.codeId),
        router: new ContractInstantiationInfo(router_upload.originalChecksum, router_upload.codeId)
    }
}

export function create_rand_base64(): string {
    const rand_bytes = Random.getBytes(32)
    return Buffer.from(rand_bytes).toString('base64')
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

export function read_config(
    chain: string,
    on_file_not_found: (file: string) => void
): any {
    const file = resolve(`./settings/${chain}.json`)

    try {
        return JSON.parse(readFileSync(file).toString())   
    } catch(e: any) {
        if (e.message.includes('no such file or directory')) {
            on_file_not_found(file)
            
            return
        }

        throw e
    }
}

export async function create_account(): Promise<ScrtAccount> {
    const key_pair = EnigmaUtils.GenerateNewKeyPair()
    const mnemonic = Bip39.encode(key_pair.privkey).toString()
    const pen = await Secp256k1Pen.fromMnemonic(mnemonic)

    const pubkey = encodeSecp256k1Pubkey(pen.pubkey)

    return {
        mnemonic,
        address: pubkeyToAddress(pubkey, 'secret')
    }
}

import { Address } from './core'
import { SigningCosmWasmClient, CosmWasmClient } from 'secretjs'

export class SmartContract {
    constructor(
        readonly address: Address,
        readonly signing_client: SigningCosmWasmClient,
        readonly client?: CosmWasmClient | undefined
    ) { }

    protected query_client(): CosmWasmClient | SigningCosmWasmClient {
        if (this.client !== undefined) {
            return this.client
        }

        return this.signing_client
    }
}

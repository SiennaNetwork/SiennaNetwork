import { Address, Fee, Coin, create_fee, create_entropy, ViewingKey, decode } from './core'
import { SigningCosmWasmClient, CosmWasmClient, ExecuteResult } from 'secretjs'

export abstract class SmartContract<E extends Executor, Q extends Querier> {
    public readonly execute_client?: SigningCosmWasmClient
    public readonly query_client: CosmWasmClient | SigningCosmWasmClient

    constructor(
        readonly address: Address,
        execute_client?: SigningCosmWasmClient,
        query_client?: CosmWasmClient
    ) { 
        if (execute_client) {
            this.execute_client = execute_client

            if (query_client) {
                this.query_client = query_client
            } else {
                this.query_client = execute_client
            }
        } else if (query_client) {
            this.query_client = query_client
        } else {
            throw new Error('At least one type of client is expected.')
        }
    }

    abstract exec(fee?: Fee, memo?: string): E
    abstract query(): Q
}

export abstract class Executor {
    protected readonly client: SigningCosmWasmClient

    constructor(
        protected address: Address,
        client?: SigningCosmWasmClient,
        protected fee?: Fee,
        protected memo?: string,
    ) {
        if (!client) {
            throw new Error('No instance of SigningCosmWasmClient was provided.')
        }

        this.client = client
    }

    protected async run(msg: object, defaultGas: string, funds?: Coin[]): Promise<ExecuteResult> {
        const fee = this.fee || create_fee(defaultGas)

        return this.client.execute(this.address, msg, this.memo || "", funds, fee)
    }

    async create_viewing_key(
        padding?: string | null,
    ): Promise<string> {
        const msg = {
            create_viewing_key: {
                entropy: create_entropy(),
                padding,
            },
        }

        const result = await this.run(
            msg,
            "200000",
        )

        return ((decode(result.data) as any).create_viewing_key as any).key as string
    }

    async set_viewing_key(
        key: ViewingKey,
        padding?: string | null,
    ): Promise<string> {
        const msg = {
            set_viewing_key: {
                key,
                padding,
            },
        }

        const result = await this.run(
            msg,
            "200000",
        )

        return ((decode(result.data) as any).set_viewing_key as any).key as string
    }
}

export abstract class Querier {
    constructor(
        protected address: Address,
        protected client: CosmWasmClient | SigningCosmWasmClient
    ) { }

    protected async run(msg: object): Promise<any> {
        return this.client.queryContractSmart(this.address, msg)
    }
}

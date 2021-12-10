import { RestClient, BroadcastMode, TxsResponse } from 'secretjs'

const MISSING_VALUE = 'N/A'

type TxHash = string

export interface GasUsage {
    name: string,
    gas_wanted: string,
    gas_used: string
}

class NamedTxResponse {
    constructor(
        readonly name: string,
        readonly data: TxsResponse
    ) { }

    gas(): GasUsage {
        return {
            name: this.name,
            gas_wanted: this.data.gas_wanted || MISSING_VALUE,
            gas_used: this.data.gas_used || MISSING_VALUE
        }
    }
}

export class TxAnalytics {
    private readonly outstanding = new Map<TxHash, string>()
    private readonly resolved = new Map<TxHash, NamedTxResponse>()
    private readonly rest: RestClient

    constructor(apiUrl: string) {
        this.rest = new RestClient(apiUrl, BroadcastMode.Block)
    }

    add_tx(hash: TxHash, name?: string | undefined) {
        if (name === undefined) {
            name = hash
        }

        this.outstanding.set(hash, name)
    }

    async get_gas_report(): Promise<GasUsage[]> {
        await this.get_txs()

        const values = Array.from(this.resolved.values())

        return values.map(val => {
            return val.gas()
        });
    }

    async get_gas_usage(hash: TxHash): Promise<GasUsage> {
        let resp = this.resolved.get(hash)

        if (resp === undefined) {
            const name = this.outstanding.get(hash)

            const data = await this.query_tx(hash)

            if (name) {
                this.outstanding.delete(hash)
                resp = new NamedTxResponse(name, data)
            } else {
                resp = new NamedTxResponse(hash, data)
            }

            this.resolved.set(hash, resp)
        }

        return resp.gas()
    }

    private async get_txs() {
        if (this.outstanding.size === 0) {
            return
        }

        for(let val of this.outstanding.entries()) {
            const data = await this.query_tx(val[0])

            this.resolved.set(
                val[0],
                new NamedTxResponse(val[1], data)
            )
        }

        this.outstanding.clear()
    }

    private async query_tx(hash: TxHash): Promise<TxsResponse> {
        return await this.rest.get(`/txs/${hash}`) as TxsResponse
    }
}

import { ExecuteResult, RestClient, BroadcastMode, TxsResponse } from 'secretjs'

export interface GasUsage {
    name: string,
    gas_wanted: string,
    gas_used: string
}

interface NamedTx {
    name: string,
    tx: ExecuteResult
}

const MISSING_VALUE = 'N/A'

export class TxAnalytics {
    private readonly txs: Map<string, NamedTx> = new Map<string, NamedTx>()
    private readonly rest: RestClient
    private cache: TxsResponse[] | null = null

    constructor(apiUrl: string) {
        this.rest = new RestClient(apiUrl, BroadcastMode.Block)
    }

    add_tx(name: string, tx: ExecuteResult) {
        this.txs.set(tx.transactionHash, {
            name,
            tx
        })

        this.cache = null
    }

    async get_gas_usage(): Promise<GasUsage[]> {
        const result = await this.get_txs()

        return result.map(tx => {
            const named_tx = this.txs.get(tx.txhash)

            return { 
                name: named_tx ? named_tx.name : tx.txhash,
                gas_wanted: tx.gas_wanted || MISSING_VALUE,
                gas_used: tx.gas_used || MISSING_VALUE
            }
        });
    }

    private async get_txs(): Promise<TxsResponse[]> {
        if (this.cache != null) {
            return this.cache
        }

        const values = Array.from(this.txs.values())

        const result = await Promise.all(
            values
                .map(x => x.tx.transactionHash)
                .map(id => this.rest.get(`/txs/${id}`) as any)
        )

        this.cache = result

        return result
    }
}

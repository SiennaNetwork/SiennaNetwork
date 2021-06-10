import { RestClient, BroadcastMode } from 'secretjs';
const MISSING_VALUE = 'N/A';
export class TxAnalytics {
    constructor(apiUrl) {
        this.txs = new Map();
        this.cache = null;
        this.rest = new RestClient(apiUrl, BroadcastMode.Block);
    }
    add_tx(name, tx) {
        this.txs.set(tx.transactionHash, {
            name,
            tx
        });
        this.cache = null;
    }
    async get_gas_usage() {
        const result = await this.get_txs();
        return result.map(tx => {
            const named_tx = this.txs.get(tx.txhash);
            return {
                name: named_tx ? named_tx.name : tx.txhash,
                gas_wanted: tx.gas_wanted || MISSING_VALUE,
                gas_used: tx.gas_used || MISSING_VALUE
            };
        });
    }
    async get_txs() {
        if (this.cache != null) {
            return this.cache;
        }
        const values = Array.from(this.txs.values());
        const result = await Promise.all(values
            .map(x => x.tx.transactionHash)
            .map(id => this.rest.get(`/txs/${id}`)));
        this.cache = result;
        return result;
    }
}
//# sourceMappingURL=tx_analytics.js.map
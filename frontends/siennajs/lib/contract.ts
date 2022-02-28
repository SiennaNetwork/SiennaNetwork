import { Address, Fee, create_fee } from './core'
import { SecretNetworkClient, Tx, MsgExecuteContract, Coin } from 'secretjs'

export abstract class SmartContract<E extends Executor, Q extends Querier> {
    constructor(
        readonly address: Address,
        readonly client: SecretNetworkClient
    ) { }

    abstract exec(fee?: Fee, memo?: string): E
    abstract query(): Q
}

export abstract class Executor {
    constructor(
        protected address: Address,
        readonly client: SecretNetworkClient,
        public fee?: Fee,
        public memo?: string,
    ) { }

    protected async run(msg: object, defaultGas: string, funds?: Coin[]): Promise<Tx> {
        const fee = this.fee || create_fee(defaultGas)
        const execute_msg = new MsgExecuteContract({
            sender: this.client.address,
            contract: this.address,
            msg,
            sentFunds: funds
        })

        return this.client.tx.broadcast([ execute_msg ], {
            memo: this.memo,
            gasLimit: parseInt(fee.gas)
        })
    }
}

export abstract class Querier {
    constructor(
        protected address: Address,
        protected client: SecretNetworkClient
    ) { }

    protected async run(msg: object): Promise<any> {
        return this.client.query.compute.queryContract({
            address: this.address,
            query: msg
        })
    }

    protected async get_height(): Promise<string | undefined> {
        const result = await this.client.query.tendermint.getLatestBlock({});
        return result.block?.header?.height;
    }
}

import { Address, TokenPair, IdoInitConfig, Pagination } from './types.js'
import { ExecuteResult, SigningCosmWasmClient } from 'secretjs'

export const FEES = {
    upload: {
        amount: [{ amount: "2000000", denom: "uscrt" }],
        gas: "2000000",
    },
    init: {
        amount: [{ amount: "500000", denom: "uscrt" }],
        gas: "500000",
    },
    exec: {
        amount: [{ amount: "600000", denom: "uscrt" }],
        gas: "600000",
    },
    send: {
        amount: [{ amount: "80000", denom: "uscrt" }],
        gas: "80000",
    },
  }

export interface GetExchangePairResponse {
    get_exchange_pair: {
        pair: TokenPair;
    }
}

export interface GetExchangeAddressResponse {
    get_exchange_address: {
        address: Address;
    }
}

export interface ListIdosResponse {
    list_idos: {
        idos: Address[];
    }
}

export class FactoryContract {
    constructor(readonly client: SigningCosmWasmClient, readonly address: Address) { }

    async create_exchange(pair: TokenPair): Promise<ExecuteResult> {
        const msg = {
            create_exchange: {
                pair
            }
        }

        return await this.client.execute(this.address, msg)
    }

    async create_ido(info: IdoInitConfig): Promise<ExecuteResult> {
        const msg = {
            create_ido: {
                info
            }
        }

        return await this.client.execute(this.address, msg)
    }
    
    async get_exchange_pair(exchange_addr: Address): Promise<GetExchangePairResponse> {
        const msg = {
            get_exchange_pair: {
                exchange_addr
            }
        }

        return await this.client.queryContractSmart(this.address, msg) as GetExchangePairResponse
    }

    async get_exchange_address(pair: TokenPair): Promise<GetExchangeAddressResponse> {
        const msg = {
            get_exchange_address: {
                pair
            }
        }

        return await this.client.queryContractSmart(this.address, msg) as GetExchangeAddressResponse
    }

    async list_idos(pagination: Pagination): Promise<ListIdosResponse> {
        const msg = {
            pagination: {
                pagination
            }
        }

        return await this.client.queryContractSmart(this.address, msg) as ListIdosResponse
    }
}

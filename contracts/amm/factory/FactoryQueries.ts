import { QueryExecutor } from '@fadroma/scrt'
export class FactoryQueries extends QueryExecutor {
  async list_exchanges (start: number, limit: number): Promise<Exchange[]> {
    const response = await this.query({ list_exchanges: { pagination: { start, limit } } })
    return response.list_exchanges.exchanges
  }
  async get_exchange_address (token_0: TokenType, token_1: TokenType) {
    const response = await this.query({ get_exchange_address: { token_0, token_1 } })
    return response.get_exchange_address
  }
}

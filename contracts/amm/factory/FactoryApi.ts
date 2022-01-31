import { QueryExecutor, TransactionExecutor } from '@hackbg/fadroma'
import { TokenType }   from './schema/handle_msg.d'
import { Exchange }    from './schema/query_response.d'
import { b64encode }   from "@waiting/base64";
import { EnigmaUtils } from "secretjs";

export class FactoryQueries extends QueryExecutor {
  async get_config (): Promise<any> {
    const { config } = await this.query({ get_config: {} })
    return config
  }
  async list_exchanges (start: number, limit: number): Promise<Exchange[]> {
    const response = await this.query({ list_exchanges: { pagination: { start, limit } } })
    return response.list_exchanges.exchanges
  }
  async get_exchange_address (token_0: TokenType, token_1: TokenType) {
    const response = await this.query({ get_exchange_address: { pair: { token_0, token_1 } } })
    return response.get_exchange_address
  }
}


export class FactoryTransactions extends TransactionExecutor {
  create_exchange (token_0: TokenType, token_1: TokenType) {
    const entropy = b64encode(EnigmaUtils.GenerateNewSeed().toString());
    return this.execute({ create_exchange: { pair: { token_0, token_1 }, entropy } })
  }
  create_launchpad (tokens: object[]) {
    return this.execute({ create_launchpad: { tokens } })
  }
}

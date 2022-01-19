import { TransactionExecutor } from '@fadroma/scrt'
export class FactoryTransactions extends TransactionExecutor {
  create_exchange (token_0: TokenType, token_1: TokenType) {
    const entropy = b64encode(EnigmaUtils.GenerateNewSeed().toString());
    return this.execute({ create_exchange: { pair: { token_0, token_1 }, entropy } })
  }
  create_launchpad (tokens: object[]) {
    return this.execute({ create_launchpad: { tokens } })
  }
}

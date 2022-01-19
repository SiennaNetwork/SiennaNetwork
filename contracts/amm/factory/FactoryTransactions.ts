import { b64encode } from "@waiting/base64";
import { TransactionExecutor } from '@fadroma/scrt'
import { TokenType } from './schema/handle_msg.d'
import { EnigmaUtils } from "secretjs/src/index.ts";

export class FactoryTransactions extends TransactionExecutor {
  create_exchange (token_0: TokenType, token_1: TokenType) {
    const entropy = b64encode(EnigmaUtils.GenerateNewSeed().toString());
    return this.execute({ create_exchange: { pair: { token_0, token_1 }, entropy } })
  }
  create_launchpad (tokens: object[]) {
    return this.execute({ create_launchpad: { tokens } })
  }
}

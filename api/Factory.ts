import { ScrtContract, loadSchemas, ContractAPIOptions } from "@fadroma/scrt";
import { TokenTypeFor_HumanAddr } from "./factory/handle_msg.d";
import { EnigmaUtils } from "secretjs/src/index.ts";
import { b64encode } from "@waiting/base64";
import { abs } from "../ops/index";
import { randomHex } from "@fadroma/tools";

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./factory/init_msg.json",
  queryMsg: "./factory/query_msg.json",
  queryAnswer: "./factory/query_response.json",
  handleMsg: "./factory/handle_msg.json",
});

export class Factory extends ScrtContract {
  constructor(options: ContractAPIOptions = {}) {
    super({ ...options, schema });
    
    if (options.initMsg) {
      this.init.msg = options.initMsg;
    }
    if (options.label) {
      this.init.label = options.label;
    }
  }

  code = { ...this.code, workspace: abs(), crate: "factory" };

  init = {
    ...this.init,
    label: "SiennaAMMFactory",
    msg: {
      get prng_seed() {
        return randomHex(36);
      },
      exchange_settings: {
        swap_fee: { nom: 28, denom: 1000 },
        sienna_fee: { nom: 2, denom: 10000 },
        sienna_burner: null,
      },
    },
  };

  /**
   * Create launchpad contract
   * 
   * @param {object[]} tokens 
   * @param {Agent} agent 
   * @returns 
   */
  createLaunchpad(tokens: object[], agent = this.instantiator) {
    return this.tx.create_launchpad({
      tokens,
    }, agent);
  }

  /**
   * 
   * @param {object} token_0 
   * @param {object} token_1 
   * @param {Agent} agent 
   * @returns 
   */
  createExchange(token_0: any, token_1: any, agent = this.instantiator) {
    return this.execute(
      "create_exchange",
      {
        pair: {
          token_0: { custom_token: token_0 },
          token_1: { custom_token: token_1 },
        },
        entropy: b64encode(EnigmaUtils.GenerateNewSeed().toString()),
      },
      "",
      [],
      undefined,
      agent
    );
  }
  
  /**
   * List all the exchanges present in the factory
   */
  listExchanges() {
    this.q.listExchanges({ pagination: { start: 0, limit: 100 } });
  }
}

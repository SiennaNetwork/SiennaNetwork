import assert from 'assert'
import type { Agent } from '@fadroma/ops'
import { ScrtContract, loadSchemas, ContractAPIOptions } from "@fadroma/scrt";
import { TokenTypeFor_HumanAddr } from "./factory/handle_msg.d";
import { EnigmaUtils } from "secretjs/src/index.ts";
import { b64encode } from "@waiting/base64";
import { randomHex, Console } from "@fadroma/tools";

const console = Console(import.meta.url)

import { AMM } from './AMM'
import { AMMSNIP20, LPToken } from './SNIP20'
import { IDO } from './IDO'
import { Launchpad } from './Launchpad'

import { abs } from "../ops/index";
import { SwapRouter } from './Router';

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./factory/init_msg.json",
  queryMsg: "./factory/query_msg.json",
  queryAnswer: "./factory/query_response.json",
  handleMsg: "./factory/handle_msg.json",
});

type FactoryConstructorOptions = ContractAPIOptions & {
  admin:      Agent,
  swapConfig: any,
  EXCHANGE:   AMM,
  AMMTOKEN:   AMMSNIP20,
  LPTOKEN:    LPToken,
  IDO: IDO,
  ROUTER: SwapRouter
}

export class Factory extends ScrtContract {

  constructor(options: {
    admin?:     Agent
    prefix?:   string
    config?:    any
    EXCHANGE?:  AMM
    AMMTOKEN?:  AMMSNIP20
    LPTOKEN?:   LPToken
    IDO?:       IDO,
    ROUTER?:    SwapRouter,
    LAUNCHPAD?: Launchpad,
    codeId?: number,
    label?: string,
  } = {}) {
    super({ codeId: options.codeId, agent: options.admin, prefix: options.prefix, schema, workspace: abs() })

    Object.assign(this.init.msg, {
      ...(options.config || {}),
      admin: options.admin?.address,
    })

    const { EXCHANGE, AMMTOKEN, LPTOKEN, IDO, ROUTER, LAUNCHPAD } = options
    Object.assign(this.dependencies, { EXCHANGE, AMMTOKEN, LPTOKEN, IDO, ROUTER, LAUNCHPAD })

    const self = this
    Object.defineProperties(this.init.msg, {
      snip20_contract:    {
        enumerable: true,
        get () { return self.dependencies.AMMTOKEN.template }
      },
      pair_contract:      {
        enumerable: true,
        get () { return self.dependencies.EXCHANGE.template }
      },
      lp_token_contract:  {
        enumerable: true,
        get () { return self.dependencies.LPTOKEN.template }
      },
      ido_contract:       {
        enumerable: true,
        get () { return self.dependencies.IDO.template }
      },
      router_contract:       {
        enumerable: true,
        get () { return self.dependencies.ROUTER.template }
      },
      launchpad_contract: {
        enumerable: true,
        get () { return self.dependencies.LAUNCHPAD.template }
      },
    })

    if (options.label) {
      this.init.label = options.label;
    }
  }

  dependencies: Record<string, ScrtContract> = {}

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
   * @param {TokenTypeFor_HumanAddr} token_0 
   * @param {TokenTypeFor_HumanAddr} token_1 
   * @param {Agent} agent 
   * @returns 
   */
  async createExchange (
    token_0: TokenTypeFor_HumanAddr,
    token_1: TokenTypeFor_HumanAddr,
    agent = this.instantiator
  ) {
    const pair = { token_0, token_1 };
    const entropy = b64encode(EnigmaUtils.GenerateNewSeed().toString());
    await agent.execute(this.link, "create_exchange", { pair, entropy });
    return await this.getExchange(token_0, token_1, agent);
  }

  async getExchange (
    token_0: TokenTypeFor_HumanAddr,
    token_1: TokenTypeFor_HumanAddr,
    agent = this.instantiator
  ) {
    const pair = { token_0, token_1 };
    const {get_exchange_address:{address:exchange_address}} =
      await agent.query(this.link, "get_exchange_address", { pair })
    const exchange = AMM.attach(exchange_address, undefined, agent)
    const {pair_info:{liquidity_token:lp_token}} = await exchange.pairInfo();
    return { exchange: exchange.link, lp_token, token_0, token_1 }
  }

  listExchanges = async () => {
    const result = []

    const limit = 30
    let start = 0

    while(true) {
      const response = await this.q.listExchanges({ pagination: { start, limit } })
      const list = response.list_exchanges.exchanges

      if (list.length == 0)
        break

      result.push.apply(result, list)
      start += limit
    }
    
    return result
  }
}

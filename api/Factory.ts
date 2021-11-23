import assert from 'assert'
import type { Agent } from '@fadroma/ops'
import { ScrtContract, loadSchemas, ContractAPIOptions } from "@fadroma/scrt";
import { TokenTypeFor_HumanAddr } from "./factory/handle_msg.d";
import { EnigmaUtils } from "secretjs/src/index.ts";
import { b64encode } from "@waiting/base64";
import { randomHex } from "@fadroma/tools";

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
    const args = {
      pair:    { token_0, token_1 },
      entropy: b64encode(EnigmaUtils.GenerateNewSeed().toString()),
    };

    const result = await this.execute(
      "create_exchange", args, "", [], undefined, agent
    );

    const {logs:[{events:[_,{attributes}]}]} = result

    const [ {value:contract_address_1}
          , {value:action_1}
          , {value:pair}
          , {value:contract_address_2}
          , {value:created_exchange_address}
          , {value:contract_address_3}
          , {value:register_status_1}
          , {value:contract_address_4}
          , {value:register_status_2}
          , {value:contract_address_5}
          , {value:token_address}
          , {value:token_code_hash}
          , {value:contract_address_6}
          , {value:liquidity_token_addr}
          , {value:contract_address_7}
          , {value:register_status_3}
          , {value:contract_address_8}
          , {value:action_2}
          , {value:address} ] = attributes

    console.log({token_0, token_1})

    const title =
      `Token 1: ${token_0.custom_token.contract_addr} \n `+
      `Token 2: ${token_1.custom_token.contract_addr}`

    for (const [a, b] of [
      [pair, title],

      [contract_address_1, this.address],
      [contract_address_8, this.address],

      [action_1, 'create_exchange'],

      [action_2, 'register_exchange'],

      [register_status_1, 'success'],
      [register_status_2, 'success'],
      [register_status_3, 'success'],

      [contract_address_2, created_exchange_address],
      [address,            created_exchange_address],

      [contract_address_3, token_0.custom_token.contract_addr],
      [contract_address_4, token_1.custom_token.contract_addr],

      [contract_address_5,   token_address],
      [liquidity_token_addr, token_address],

      [token_address,      liquidity_token_addr],
      [contract_address_7, liquidity_token_addr],
      [token_code_hash,    this.dependencies.LPTOKEN.codeHash],

      [contract_address_6, created_exchange_address],
    ]) {
      assert.strictEqual(
        a, b,
        '@sienna/api (Factory#createExchange): '+
        'Could not parse logs from exchange pair creation. ' +
        'Parsing them is fragile - it depends on a particular order. ' +
        'Has the behavior of the Factory changed?'
      )
    }

    return {
      exchange: { address: created_exchange_address },
      lp_token: { address: liquidity_token_addr, code_hash: token_code_hash },
      token_0,
      token_1,
    }
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

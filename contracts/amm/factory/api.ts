import type { IAgent } from "@fadroma/ops";
import { ScrtContract_1_2, loadSchemas, ContractAPIOptions } from "@fadroma/scrt";
import { randomHex } from "@hackbg/tools";

import { b64encode } from "@waiting/base64";
import { EnigmaUtils } from "secretjs/src/index.ts";

import { AMMContract        } from "@sienna/exchange";
import { AMMSNIP20Contract  } from "@sienna/amm-snip20";
import { LPTokenContract    } from "@sienna/lp-token";
import { IDOContract        } from "@sienna/ido";
import { LaunchpadContract  } from "@sienna/launchpad";

import { workspace } from "@sienna/settings";
import { TokenTypeFor_HumanAddr } from "./schema/handle_msg.d";

export const schema = loadSchemas(import.meta.url, {
  initMsg:     "./schema/init_msg.json",
  queryMsg:    "./schema/query_msg.json",
  queryAnswer: "./schema/query_response.json",
  handleMsg:   "./schema/handle_msg.json",
});

export type FactoryOptions = ContractAPIOptions & {
  admin?:     IAgent,
  config?:    any,
  EXCHANGE?:  AMMContract,
  AMMTOKEN?:  AMMSNIP20Contract,
  LPTOKEN?:   LPTokenContract,
  IDO?:       IDOContract
  LAUNCHPAD?: LaunchpadContract
  ROUTER?:    SwapRouterContract
}

export class FactoryContract extends ScrtContract_1_2 {

  static attach = (
    address:  string,
    codeHash: string,
    agent:    IAgent
  ) => {
    const instance = new FactoryContract({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }

  constructor(options: FactoryOptions = {}) {
    super({
      codeId: options.codeId,
      agent:  options.admin,
      prefix: options.prefix,
      schema,
      workspace
    })

    Object.assign(this.init.msg, {
      ...(options.config || {}),
      admin: options.admin?.address,
    })

    const { EXCHANGE, AMMTOKEN, LPTOKEN, IDO, ROUTER, LAUNCHPAD } = options
    Object.assign(this.dependencies, { EXCHANGE, AMMTOKEN, LPTOKEN, IDO, ROUTER, LAUNCHPAD })

    const self = this
    Object.defineProperties(this.init.msg, {
      snip20_contract:    { enumerable: true, get () { return self.dependencies.AMMTOKEN.template  } },
      pair_contract:      { enumerable: true, get () { return self.dependencies.EXCHANGE.template  } },
      lp_token_contract:  { enumerable: true, get () { return self.dependencies.LPTOKEN.template   } },
      ido_contract:       { enumerable: true, get () { return self.dependencies.IDO.template       } },
      launchpad_contract: { enumerable: true, get () { return self.dependencies.LAUNCHPAD.template } },
    })

    if (options.label) {
      this.init.label = options.label;
    }
  }

  readonly dependencies: Record<string, ScrtContract> = {}

  code = { ...this.code, workspace, crate: "factory" };

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
   * @param {IAgent} agent
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
   * @param {IAgent} agent
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

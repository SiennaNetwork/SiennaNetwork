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
  IDO:        IDO
}

export class Factory extends ScrtContract {

  constructor(options: {
    admin:     Agent
    config:    any
    EXCHANGE:  AMM
    AMMTOKEN:  AMMSNIP20
    LPTOKEN:   LPToken
    IDO:       IDO
    LAUNCHPAD: Launchpad
  }) {
    super({ agent: options.admin, schema })
    Object.assign(this.init.msg, {
      ...options.config,
      admin: options.admin.address,
    })
    Object.defineProperties(this.init.msg, {
      snip20_contract:    { enumerable: true, get () { return options.AMMTOKEN.template } },
      pair_contract:      { enumerable: true, get () { return options.EXCHANGE.template } },
      lp_token_contract:  { enumerable: true, get () { return options.LPTOKEN.template } },
      ido_contract:       { enumerable: true, get () { return options.IDO.template } },
      launchpad_contract: { enumerable: true, get () { return options.LAUNCHPAD.template } },
    })
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
  createExchange = (token_0: any, token_1: any, agent = this.instantiator) =>
    this.execute(
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
  /*this.tx.createExchange({
    pair:    { token_0, token_1 },
    entropy: b64encode(EnigmaUtils.GenerateNewSeed().toString()) })*/
  listExchanges = () =>
    this.q.listExchanges({ pagination: { start: 0, limit: 100 } });
}

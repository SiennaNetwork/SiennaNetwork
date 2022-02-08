import { Console, bold, timestamp, randomHex } from "@hackbg/fadroma"

const console = Console('@sienna/exchange')

import { InitMsg } from './schema/init_msg.d'
import { TokenType, TokenPair, ContractLink } from './schema/query_msg_response.d'
import { AMMSNIP20Contract } from '@sienna/amm-snip20'
import { LPTokenContract } from '@sienna/lp-token'
import getSettings, { workspace } from '@sienna/settings'

import { Scrt_1_2 } from "@hackbg/fadroma"
import type { AMMVersion, ExchangeInfo } from './ExchangeClient'
import { AMMExchangeClient } from './ExchangeClient'
export { AMMExchangeClient, AMMVersion, ExchangeInfo }
export abstract class AMMExchangeContract extends Scrt_1_2.Contract<AMMExchangeClient> {

  name   = 'AMM.Exchange'
  abstract readonly version: AMMVersion
  source = { workspace, crate: 'exchange' }
  Client = AMMExchangeClient

  initMsg?: InitMsg = {
    callback:          { contract: null, msg: null },
    entropy:           null,
    factory_info:      { address: null, code_hash: null },
    lp_token_contract: { id: null, code_hash: null },
    pair:              null,
    prng_seed:         randomHex(36),
  }

  static "v1" = class AMMExchangeContract_v1 extends AMMExchangeContract {
    name   = 'AMM[v1].Exchange'
    version = "v1" as AMMVersion
    source  = { workspace, crate: 'exchange', ref: 'a99d8273b4' }
  }

  static "v2" = class AMMExchangeContract_v1 extends AMMExchangeContract {
    name   = 'AMM[v2].Exchange'
    version = "v2" as AMMVersion
  }

  /** Procedure. Deploy a new exchange.
    * If the exchange already exists, do nothing.
    * Factory doesn't allow 2 identical exchanges to exist anyway.
    * (as compared by TOKEN0 and TOKEN1). */
  static deploy = deployAMMExchange

  /** Since exchange and LP token are deployed through the factory
    * and not though Fadroma Deploy, we need to manually save their
    * addresses in the Deployment. */ 
  static save = saveAMMExchange

  /** Command. */
  static deployMany = deployAMMExchanges

  /** Command. */
  static redeployMany = redeployAMMExchanges

}

import { colors, print } from '@hackbg/fadroma'

export async function printExchanges (EXCHANGES?: any[]) {
  if (!EXCHANGES) {
    console.info('No exchanges found.')
    return
  }
  for (const { name, EXCHANGE, TOKEN_0, TOKEN_1, LP_TOKEN } of EXCHANGES) {
    const { codeId, codeHash, address } = EXCHANGE
    console.info(
      ' ', bold(colors.inverse(name)).padEnd(30), // wat
      `(code id ${bold(String(codeId))})`.padEnd(34), bold(address)
    )
    await print.token(TOKEN_0)
    await print.token(TOKEN_1)
    await print.token(LP_TOKEN)
  }
}

async function deployAMMExchange (options) {
  const {
    agent, deployment, run,
    TOKENS = await run(AMMSNIP20Contract.getSupportedTokens),
    FACTORY,
    name,
    ammVersion
  } = options

  const factory   = FACTORY.client(agent)
  const inventory = await factory.getContracts()
  const { token0, token1 } = await run(AMMSNIP20Contract.tokensFromName, { TOKENS, name })

  try {
    const { EXCHANGE, LP_TOKEN } = await factory.getExchange(token0, token1)
    EXCHANGE.prefix = LP_TOKEN.prefix = deployment.prefix
    console.info(`${bold(name)}: Already exists.`)
    return { EXCHANGE, LP_TOKEN }
  } catch (e) {
    if (e.message.includes("Address doesn't exist in storage")) {
      await factory.createExchange(token0, token1)
      const exchange = await factory.getExchange(token0, token1)
      return saveAMMExchange({ deployment, ammVersion, inventory, exchange })
    } else {
      console.error(e)
      throw new Error(`${bold(`Factory::GetExchange(${name})`)}: not found (${e.message})`)
    }
  }

}

import { MigrationContext } from '@hackbg/fadroma'
import { FactoryClient } from '@sienna/api'
async function deployAMMExchanges (options: MigrationContext & {
  settings: { swapPairs: string[] }
  TOKENS:     any,
  FACTORY:    FactoryClient,
  ammVersion: AMMVersion
}) {
  const {
    run, agent, deployment,
    settings: { swapPairs } = getSettings(agent.chain.id),
    TOKENS = await run(AMMSNIP20Contract.getSupportedTokens),
    FACTORY,
    ammVersion
  } = options
  if (swapPairs.length > 0) {

    const createdPairs = []

    await agent.bundle().wrap(async bundle=>{
      const agent = FACTORY.agent
      FACTORY.agent = bundle
      const factory = new FactoryClient({...FACTORY})
      for (const name of swapPairs) {
        const { token0, token1 } = await run(AMMSNIP20Contract.tokensFromName, { TOKENS, name })
        await factory.createExchange(token0, token1)
        createdPairs.push([token0, token1])
      }
      FACTORY.agent = agent
    })

    const { EXCHANGES } = await run(saveCreatedPairs, { FACTORY, ammVersion, createdPairs })

    return {
      EXCHANGES: EXCHANGES.map(EXCHANGE=>EXCHANGE.EXCHANGE),
      LP_TOKENS: EXCHANGES.map(EXCHANGE=>EXCHANGE.LP_TOKEN)
    }

  }
}

async function saveCreatedPairs ({ FACTORY, deployment, ammVersion, createdPairs }) {
  const inventory = await FACTORY.getContracts()
  const EXCHANGES = await Promise.all(createdPairs.map(async ([token0, token1])=>{
    const exchange = await FACTORY.getExchange(token0, token1)
    return saveAMMExchange({
      deployment,
      ammVersion,
      inventory,
      exchange
    })
  }))
  return { EXCHANGES }
}

async function redeployAMMExchanges (options) {

  const {
    agent, deployment,
    NEW_FACTORY,
    OLD_EXCHANGES = [],
    ammVersion
  } = options

  // create them in one go
  let NEW_EXCHANGES = []
  await agent.bundle(async agent=>{
    const bundled = NEW_FACTORY.client(agent)
    for (const { name, TOKEN_0, TOKEN_1 } of (OLD_EXCHANGES||[])) {
      const exchange = await bundled.createExchange(TOKEN_0, TOKEN_1)
      NEW_EXCHANGES.push([TOKEN_0, TOKEN_1])
    }
  })

  // get them
  const factory   = NEW_FACTORY.client(agent)
  const inventory = await NEW_FACTORY.client(agent).getContracts()
  NEW_EXCHANGES = await Promise.all(NEW_EXCHANGES.map(async ([TOKEN_0, TOKEN_1])=>{
    const exchange = await factory.getExchange(TOKEN_0, TOKEN_1)
    return saveAMMExchange({ deployment, ammVersion, inventory, exchange })
  }))

  return { NEW_EXCHANGES }

}

async function saveAMMExchange ({
  deployment,
  ammVersion,
  inventory: {
    pair_contract: { id: ammId, code_hash: ammHash },
    lp_token_contract: { id: lpId }
  },
  exchange: { name, raw, EXCHANGE, LP_TOKEN, TOKEN_0, TOKEN_1 }
}) {
  //console.info(bold(`Deployed AMM exchange`), EXCHANGE.address)
  deployment.add(`AMM[${ammVersion}].${name}`, {
    ...raw,
    codeId:   ammId,
    codeHash: ammHash,
    address:  raw.exchange.address,
  })
  //console.info(bold(`Deployed LP token`), LP_TOKEN.address)
  deployment.add(`AMM[${ammVersion}].${name}.LP`, {
    ...raw,
    codeId:   lpId,
    codeHash: raw.lp_token.code_hash,
    address:  raw.lp_token.address
  })
  EXCHANGE.prefix = LP_TOKEN.prefix = deployment.prefix
  return { name, raw, EXCHANGE, LP_TOKEN, TOKEN_0, TOKEN_1 }
}

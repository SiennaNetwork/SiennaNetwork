import { Agent, Console, bold, timestamp, randomHex, Scrt_1_2, SNIP20Contract } from "@hackbg/fadroma"
import { InitMsg } from './schema/init_msg.d'
import { AMMTransactions, AMMQueries } from './ExchangeClient'
import { TokenType, TokenPair, ContractLink } from './schema/query_msg_response.d'
import { AMMSNIP20Contract } from '@sienna/amm-snip20'
import { LPTokenContract } from '@sienna/lp-token'
import { workspace } from '@sienna/settings'

const console = Console('@sienna/exchange')

/** An exchange is an interaction between 4 contracts. */
export type ExchangeInfo = {
  /** Shorthand to refer to the whole group. */
  name?: string
  /** One token. */
  TOKEN_0:  SNIP20Contract|string,
  /** Another token. */
  TOKEN_1:  SNIP20Contract|string,
  /** The automated market maker/liquidity pool for the token pair. */
  EXCHANGE: AMMExchangeContract,
  /** The liquidity provision token, which is minted to stakers of the 2 tokens. */
  LP_TOKEN: LPTokenContract,
  /** The bare-bones data needed to retrieve the above. */
  raw:      any
}

export class AMMExchangeContract extends Scrt_1_2.Contract<AMMTransactions, AMMQueries> {

  workspace = workspace
  crate     = 'exchange'
  name      = 'AMM.Exchange'
  initMsg?: InitMsg = {
    callback:          { contract: null, msg: null },
    entropy:           null,
    factory_info:      { address: null, code_hash: null },
    lp_token_contract: { id: null, code_hash: null },
    pair:              null,
    prng_seed:         randomHex(36),
  }
  Transactions = AMMTransactions
  Queries      = AMMQueries
  token_0?: TokenType
  token_1?: TokenType
  lpToken?: SNIP20Contract
  constructor (options) {
    super(options)
    const { version } = options||{}
    if (version === 'v1') {
      this.ref    = 'a99d8273b4'
      this.suffix = `@v1+${timestamp()}`
    } else if (version === 'v2') {
      this.suffix = `@v2+${timestamp()}`
    } else {
      /* nop */
    }
  }
  get info (): Promise<any/*ExchangeInfo*/> {
    throw new Error('todo')
  }
  pairInfo = (): Promise<{ pair: TokenPair, liquidity_token: ContractLink }> => {
    return this.q().pair_info()
  }

  /** Command. Deploy a new exchange.
    * If the exchange already exists, do nothing.
    * Factory doesn't allow 2 identical exchanges to exist anyway.
    * (as compared by TOKEN0 and TOKEN1). */
  static deploy = async function deployAMMExchange ({
    agent, deployment,
    FACTORY, TOKENS, name, ammVersion
  }) {
    console.info(bold(`Deploying AMM exchange`), name)
    const [tokenName0, tokenName1] = name.split('-')

    if (!TOKENS[tokenName0]) throw new Error(
      `Missing token ${tokenName0}; available: ${Object.keys(TOKENS).join(' ')}`
    )
    const token0 = TOKENS[tokenName0].asCustomToken

    if (!TOKENS[tokenName1]) throw new Error(
      `Missing token ${tokenName1}; available: ${Object.keys(TOKENS).join(' ')}`
    )
    const token1 = TOKENS[tokenName1].asCustomToken

    const factory = FACTORY.client(agent)
    try {
      const { EXCHANGE, LP_TOKEN } = await factory.getExchange(token0, token1)
      EXCHANGE.prefix = LP_TOKEN.prefix = deployment.prefix
      console.info(`${bold(name)}: Already exists.`)
      return { EXCHANGE, LP_TOKEN }
    } catch (e) {
      if (e.message.includes("Address doesn't exist in storage")) {
        return saveExchange(
          { deployment, ammVersion },
          await factory.getContracts(),
          await factory.createExchange(token0, token1)
        )
      } else {
        console.error(e)
        throw new Error(`${bold(`Factory::GetExchange(${name})`)}: not found (${e.message})`)
      }
    }
  }

  static async getExchange (
    agent:   Agent,
    address: string,
    token_0: SNIP20Contract|TokenType,
    token_1: SNIP20Contract|TokenType
  ): Promise<ExchangeInfo> {

    const EXCHANGE = new AMMExchangeContract({
      chain: agent.chain,
      agent,
      address,
      codeHash: await agent.getCodeHash(address),
      codeId:   await agent.getCodeId(address),
    })

    const getTokenName = async TOKEN => {
      let TOKEN_NAME: string
      if (TOKEN instanceof SNIP20Contract) {
        const TOKEN_INFO = await TOKEN.q(agent).tokenInfo()
        return TOKEN_INFO.symbol
      } else {
        return 'SCRT'
      }
    }

    const TOKEN_0      = SNIP20Contract.fromTokenSpec(agent, token_0)
    const TOKEN_0_NAME = await getTokenName(TOKEN_0)

    const TOKEN_1      = SNIP20Contract.fromTokenSpec(agent, token_1)
    const TOKEN_1_NAME = await getTokenName(TOKEN_1)

    const name = `${TOKEN_0_NAME}-${TOKEN_1_NAME}`

    const { liquidity_token } = await EXCHANGE.pairInfo()

    const LP_TOKEN = new LPTokenContract({
      chain: agent.chain,
      agent,
      address:  liquidity_token.address,
      codeHash: liquidity_token.code_hash,
      codeId:   await agent.getCodeId(liquidity_token.address),
    })

    return {
      raw: { // no methods, just data
        exchange: { address: EXCHANGE.address },
        lp_token: { address: LP_TOKEN.address, code_hash: LP_TOKEN.codeHash },
        token_0,
        token_1,
      },
      name,     // The human-friendly name of the exchange
      EXCHANGE, // The exchange contract
      LP_TOKEN, // The LP token contract
      TOKEN_0,  // One token of the pair
      TOKEN_1,  // The other token of the pair
    }

  }
}

/** Since exchange and LP token are deployed through the factory
  * and not though Fadroma Deploy, we need to manually save their
  * addresses in the Deployment. */ 
export function saveExchange (
  { deployment, ammVersion },
  { pair_contract: { id: ammId, code_hash: ammHash }, lp_token_contract: { id: lpId } },
  { name, raw, EXCHANGE, LP_TOKEN, TOKEN_0, TOKEN_1 }
) {
  console.info(bold(`Deployed AMM exchange`), EXCHANGE.address)
  deployment.add(`AMM[${ammVersion}].${name}`, {
    ...raw,
    codeId:   ammId,
    codeHash: ammHash,
    address:  raw.exchange.address,
  })
  console.info(bold(`Deployed LP token`), LP_TOKEN.address)
  deployment.add(`AMM[${ammVersion}].${name}.LP`, {
    ...raw,
    codeId:   lpId,
    codeHash: raw.lp_token.code_hash,
    address:  raw.lp_token.address
  })
  EXCHANGE.prefix = LP_TOKEN.prefix = deployment.prefix
  return { name, raw, EXCHANGE, LP_TOKEN, TOKEN_0, TOKEN_1 }
}

import { colors, printToken } from '@hackbg/fadroma'
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
    await printToken(TOKEN_0)
    await printToken(TOKEN_1)
    await printToken(LP_TOKEN)
  }
}

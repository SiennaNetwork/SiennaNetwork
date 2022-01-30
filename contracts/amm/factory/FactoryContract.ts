import {
  Scrt_1_2, ContractInfo, Agent, MigrationContext,
  randomHex, colors, bold, Console, timestamp,
  printContract, printToken, printExchanges, printContracts
} from "@hackbg/fadroma"

import { SNIP20Contract } from '@fadroma/snip20'

import { AMMContract, ExchangeInfo } from "@sienna/exchange"
import { AMMSNIP20Contract, deployPlaceholders } from "@sienna/amm-snip20"
import { LPTokenContract } from "@sienna/lp-token"

import { IDOContract } from "@sienna/ido"
import { LaunchpadContract } from "@sienna/launchpad"
import { SiennaSNIP20Contract } from '@sienna/api'

import getSettings, { workspace } from '@sienna/settings'

import { InitMsg, ExchangeSettings, ContractInstantiationInfo } from './schema/init_msg.d'
import { TokenType } from './schema/handle_msg.d'
import { QueryResponse, Exchange } from './schema/query_response.d'

const console = Console('@sienna/factory')

export type FactoryInventory = {
  snip20_contract?:    ContractInstantiationInfo
  pair_contract?:      ContractInstantiationInfo
  lp_token_contract?:  ContractInstantiationInfo
  ido_contract?:       ContractInstantiationInfo
  launchpad_contract?: ContractInstantiationInfo
  router_contract?:    ContractInstantiationInfo
}

import { FactoryTransactions, FactoryQueries } from './FactoryApi'
export class FactoryContract extends Scrt_1_2.Contract<FactoryTransactions, FactoryQueries> {
  crate        = 'factory'
  name         = 'SiennaAMMFactory'
  version      = 'v2'
  Transactions = FactoryTransactions
  Queries      = FactoryQueries
  constructor (options) {
    super(options)
    const { version } = options
    if (version === 'v1') {
      this.ref    = 'a99d8273b4'
      this.suffix = `@v1+${timestamp()}`
    } else if (version === 'v2') {
      this.suffix = `@v2+${timestamp()}`
    } else {
      /* nop */
    }
  }
  /** Return the collection of contract templates
    * (`{ id, code_hash }` structs) that the factory
    * uses to instantiate contracts. */
  getContracts (): Promise<FactoryInventory> {
    // type kludge!
    if (this.address) {
      // If this contract has an address query this from the contract state
      return (this.q().get_config()).then((config: FactoryInventory)=>{
        return {
          snip20_contract:    config.snip20_contract,
          pair_contract:      config.pair_contract,
          lp_token_contract:  config.lp_token_contract,
          ido_contract:       config.ido_contract,
          launchpad_contract: config.launchpad_contract,
        }
      })
    } else {
      throw new Error('not deployed yet')
    }
  }
  get exchanges (): Promise<ExchangeInfo[]> {
    return this.listExchanges().then(exchanges=>{
      return Promise.all(
        exchanges.map(({ pair: { token_0, token_1 } }) => {
          return this.getExchange(token_0, token_1)
        })
      )
    })
  }
  /** Get the full list of raw exchange info from the factory. */
  async listExchanges (): Promise<Exchange[]> {
    const result: Exchange[] = []
    const limit = 30
    let start = 0
    while (true) {
      const list = await this.q().list_exchanges(start, limit)
      if (list.length > 0) {
        result.push(...list)
        start += limit
      } else {
        break
      }
    }
    return result
  }
  /** Get info about an exchange. */
  async getExchange (
    token_0: TokenType,
    token_1: TokenType,
    agent = this.creator || this.agent
  ): Promise<ExchangeInfo> {
    //console.info(bold('Looking for exchange'))
    //console.info(bold('  between'), JSON.stringify(token_0))
    //console.info(bold('      and'), JSON.stringify(token_1))
    const { admin, prefix, chain } = this
    const { address } = await this.q(agent).get_exchange_address(
      token_0, token_1
    )
    // ouch, factory-created contracts have nasty labels
    const label = await agent.getLabel(address)

    const EXCHANGE = new AMMContract({
      chain,
      address,
      codeHash: await agent.getCodeHash(address),
      codeId:   await agent.getCodeId(address),
      prefix,
      agent,
    })

    const TOKEN_0 = SNIP20Contract.fromTokenSpec(agent, token_0)
    let TOKEN_0_NAME: string
    if (TOKEN_0 instanceof SNIP20Contract) {
      const TOKEN_0_INFO = await TOKEN_0.q(agent).tokenInfo()
      TOKEN_0_NAME = TOKEN_0_INFO.symbol
    } else {
      TOKEN_0_NAME = 'SCRT'
    }

    const TOKEN_1 = SNIP20Contract.fromTokenSpec(agent, token_1)
    let TOKEN_1_NAME: string
    if (TOKEN_1 instanceof SNIP20Contract) {
      const TOKEN_1_INFO = await TOKEN_1.q(agent).tokenInfo()
      TOKEN_1_NAME = TOKEN_1_INFO.symbol
    } else {
      TOKEN_1_NAME = 'SCRT'
    }

    const name = `${TOKEN_0_NAME}-${TOKEN_1_NAME}`

    const { liquidity_token } = await EXCHANGE.pairInfo()
    const LP_TOKEN = new LPTokenContract({
      admin:    this.admin,
      prefix:   this.prefix,
      chain:    this.chain,
      address:  liquidity_token.address,
      codeHash: liquidity_token.code_hash,
      codeId:   await agent.getCodeId(liquidity_token.address),
      agent
    })

    const raw = {
      exchange: {
        address: EXCHANGE.address
      },
      token_0,
      token_1,
      lp_token: {
        address:   LP_TOKEN.address,
        code_hash: LP_TOKEN.codeHash
      },
    }

    return {
      name,
      EXCHANGE, TOKEN_0, TOKEN_1, LP_TOKEN,
      raw
    }
  }

  /** Create a liquidity pool, i.e. an instance of the exchange contract. */
  async createExchange (
    token_0: SNIP20Contract|TokenType,
    token_1: SNIP20Contract|TokenType,
    agent = this.agent
  ): Promise<ExchangeInfo> {
    if (token_0 instanceof SNIP20Contract) token_0 = token_0.asCustomToken
    if (token_1 instanceof SNIP20Contract) token_1 = token_1.asCustomToken
    await this.tx(agent).create_exchange(token_0, token_1)
    return await this.getExchange(token_0, token_1, agent)
  }

  /** Create an instance of the launchpad contract. */
  createLaunchpad (
    tokens: object[],
    agent = this.agent
  ) {
    return this.tx(agent).create_launchpad(tokens)
  }

}

/** Taking a TGE deployment, add the AMM to it,
  * creating the pre-configured liquidity and reward pools. */
export async function deployAMM ({
  deployment, admin, run,
  SIENNA  = deployment.getContract(admin, SiennaSNIP20Contract, 'SiennaSNIP20'),
  version = 'v2',
}): Promise<{
  /* The newly created factory contract. */
  FACTORY:   FactoryContract
  /* Collection of tokens supported by the AMM. */
  TOKENS:    Record<string, SNIP20Contract>
  /* List of exchanges created. */
  EXCHANGES: AMMContract[]
  /* List of LP tokens created. */
  LP_TOKENS: LPTokenContract[]
}> {
  const {
    FACTORY
  } = await run(deployAMMFactory, { version })
  const {
    TOKENS, EXCHANGES, LP_TOKENS
  } = await run(deployAMMExchanges, { SIENNA, FACTORY, version })
  console.log()
  console.info(bold('Deployed AMM contracts:'))
  printContracts([FACTORY,...EXCHANGES,...LP_TOKENS])
  console.log()
  return { FACTORY, TOKENS, EXCHANGES, LP_TOKENS, }
}

Object.assign(deployAMM, { 
  v1: args => deployAMM({ ...args, version: 'v1' }),
  v2: args => deployAMM({ ...args, version: 'v2' }),
})

export const upgradeAMM = {

  async v1_to_v2 ({
    run, chain, admin, deployment, prefix,
    FACTORY = deployment.getContract(
      admin, FactoryContract, 'SiennaAMMFactory@v1'
    ),
  }) {
    console.log()

    // old
    console.info(bold('Current factory:'))
    printContract(FACTORY)

    const EXCHANGES: ExchangeInfo[] = await FACTORY.exchanges
    await printExchanges(EXCHANGES)

    // new
    const version = 'v2'
    const { FACTORY: NEW_FACTORY } = await run(deployAMMFactory, {
      version,
      copyFrom: FACTORY
    })
    printContract(NEW_FACTORY)

    const NEW_EXCHANGES = []
    for (const { name, TOKEN_0, TOKEN_1 } of EXCHANGES) {
      console.info(bold('Upgrading exchange'), name)
      NEW_EXCHANGES.push(saveExchange(
        { deployment, version },
        await FACTORY.getContracts(),
        await FACTORY.createExchange(TOKEN_0, TOKEN_1)))
    }
    await printExchanges(NEW_EXCHANGES)

    return { FACTORY: NEW_FACTORY, EXCHANGES: NEW_EXCHANGES }
  }
}

/** Deploy the Factory contract which is the hub of the AMM.
  * It needs to be passed code ids and code hashes for
  * the different kinds of contracts that it can instantiate.
  * So build and upload versions of those contracts too. */
export async function deployAMMFactory ({
  prefix, admin, chain, deployment,
  version = 'v2',
  suffix  = `@${version}+${timestamp()}`,
  copyFrom,
  initMsg = {
    admin:             admin.address,
    prng_seed:         randomHex(36),
    exchange_settings: getSettings(chain.chainId).amm.exchange_settings,
  }
}) {
  const options = { workspace, prefix, admin }
  const FACTORY   = new FactoryContract({ ...options, version, suffix })
  const LAUNCHPAD = new LaunchpadContract({ ...options })
  // launchpad is new to v2 so we build/upload it every time...
  await chain.buildAndUpload(admin, [FACTORY, LAUNCHPAD])
  const template = contract => ({
    id:        contract.codeId,
    code_hash: contract.codeHash
  })
  if (copyFrom) {
    await deployment.createContract(admin, FACTORY, {
      ...initMsg,
      ...await copyFrom.getContracts(),
      // ...because otherwise here it wouldn've be able to copy it from v1...
      launchpad_contract: template(LAUNCHPAD),
    })
  } else {
    const [EXCHANGE, AMMTOKEN, LPTOKEN, IDO] = await chain.buildAndUpload(admin, [
      new AMMContract({       ...options, version }),
      new AMMSNIP20Contract({ ...options }),
      new LPTokenContract({   ...options }),
      new IDOContract({       ...options }),
    ])
    const contracts = {
      snip20_contract:    template(AMMTOKEN),
      pair_contract:      template(EXCHANGE),
      lp_token_contract:  template(LPTOKEN),
      ido_contract:       template(IDO),
      // ...while v1 here would just ignore this config field
      launchpad_contract: template(LAUNCHPAD),
    }
    await deployment.getOrCreateContract(
      admin, FACTORY, 'SiennaAMMFactory', {
        ...initMsg,
        ...contracts
      })
  }
  console.log()
  console.info(
    bold(`Deployed factory ${version}`), FACTORY.label
  )
  printContract(FACTORY)
  return { FACTORY }
}

export async function deployAMMExchanges ({
  run, chain, admin,
  SIENNA,
  FACTORY,
  version,
  settings: { swapTokens, swapPairs } = getSettings(chain.chainId),
}) {
  // Collect referenced tokens, and created exchanges/LPs
  const TOKENS:    Record<string, SNIP20Contract> = { SIENNA }
  const EXCHANGES: AMMContract[]     = []
  const LP_TOKENS: LPTokenContract[] = []
  if (chain.isLocalnet) {
    // On localnet, deploy some placeholder tokens corresponding to the config.
    const { PLACEHOLDERS } = await run(deployPlaceholders)
    Object.assign(TOKENS, PLACEHOLDERS)
  } else {
    // On testnet and mainnet, talk to preexisting token contracts from the config.
    console.info(`Not running on localnet, using tokens from config:`)
    const tokens = {}
    for (
      const [name, {address, codeHash}] of
      Object.entries(swapTokens as Record<string, { address: string, codeHash: string }>)
    ) {
      tokens[name] = new AMMSNIP20Contract({address, codeHash, admin})
    }
    Object.assign(TOKENS, tokens)
    console.debug(bold('Tokens:'), TOKENS)
  }
  // If there are any initial swap pairs defined in the config
  if (swapPairs.length > 0) {
    for (const name of swapPairs) {
      // Call the factory to deploy an EXCHANGE for each
      const { EXCHANGE, LP_TOKEN } = await run(deployAMMExchange, {
        FACTORY, TOKENS, name, version
      })
      // And collect the results
      EXCHANGES.push(EXCHANGE)
      LP_TOKENS.push(LP_TOKEN)
    }
  }
  return { TOKENS, LP_TOKENS, EXCHANGES }
}

export async function deployAMMExchange ({
  admin, deployment,
  FACTORY, TOKENS, name, version
}) {
  console.info(
    bold(`Deploying AMM exchange`), name
  )
  const [tokenName0, tokenName1] = name.split('-')
  const token0 = TOKENS[tokenName0].asCustomToken
  const token1 = TOKENS[tokenName1].asCustomToken
  //console.info(`- Token 0: ${bold(JSON.stringify(token0))}...`)
  //console.info(`- Token 1: ${bold(JSON.stringify(token1))}...`)
  try {
    const { EXCHANGE, LP_TOKEN } = await FACTORY.getExchange(token0, token1, admin)
    console.info(`${bold(name)}: Already exists.`)
    return { EXCHANGE, LP_TOKEN }
  } catch (e) {
    if (e.message.includes("Address doesn't exist in storage")) {
      return saveExchange(
        { deployment, version },
        await FACTORY.getContracts(),
        await FACTORY.createExchange(token0, token1))
    } else {
      console.error(e)
      throw new Error(`${bold(`Factory::GetExchange(${name})`)}: not found (${e.message})`)
    }
  }
}

function saveExchange (
  { deployment, version },
  { pair_contract: { id: ammId, code_hash: ammHash }, lp_token_contract: { id: lpId } },
  { name, raw, EXCHANGE, LP_TOKEN }
) {
  console.info(bold(`Deployed AMM exchange`), EXCHANGE.address)
  deployment.save({
    ...raw,
    codeId:   ammId,
    codeHash: ammHash,
    initTx:   { contractAddress: raw.exchange.address }
  }, `SiennaSwap_${version}_${name}`)
  console.info(bold(`Deployed LP token`), LP_TOKEN.address)
  deployment.save({
    ...raw,
    codeId:   lpId,
    codeHash: raw.lp_token.code_hash,
    initTx:   { contractAddress: raw.lp_token.address }
  }, `SiennaSwap_${version}_LP-${name}`)
  return { EXCHANGE, LP_TOKEN }
}

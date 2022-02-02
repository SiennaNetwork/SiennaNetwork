import {
  Scrt_1_2, SNIP20Contract, ContractInfo, Agent, MigrationContext,
  randomHex, colors, bold, Console, timestamp,
  printContract, printToken, printContracts
} from '@hackbg/fadroma'

const console = Console('@sienna/factory')

import getSettings, { workspace } from '@sienna/settings'

import { AMMExchangeContract, ExchangeInfo, saveExchange, printExchanges } from '@sienna/exchange'
import { AMMSNIP20Contract, deployPlaceholders } from '@sienna/amm-snip20'
import { LPTokenContract } from '@sienna/lp-token'
import { IDOContract } from '@sienna/ido'
import { LaunchpadContract } from '@sienna/launchpad'
import { SiennaSNIP20Contract } from '@sienna/snip20-sienna'

import { InitMsg, ExchangeSettings, ContractInstantiationInfo } from './schema/init_msg.d'
import { TokenType } from './schema/handle_msg.d'
import { QueryResponse, Exchange } from './schema/query_response.d'

export type FactoryInventory = {
  snip20_contract?:    ContractInstantiationInfo
  pair_contract?:      ContractInstantiationInfo
  lp_token_contract?:  ContractInstantiationInfo
  ido_contract?:       ContractInstantiationInfo
  launchpad_contract?: ContractInstantiationInfo
  router_contract?:    ContractInstantiationInfo
}

import { FactoryTransactions, FactoryQueries } from './FactoryApi'
export class AMMFactoryContract extends Scrt_1_2.Contract<FactoryTransactions, FactoryQueries> {
  //workspace    = 'workspace'
  crate        = 'factory'
  Transactions = FactoryTransactions
  Queries      = FactoryQueries
  /** Command. Take the active TGE deployment, add the AMM Factory to it, use it to
    * create the configured AMM Exchange liquidity pools and their LP tokens. */
  static deployAMM = async function deployAMM ({ run, version }) {
    const { FACTORY } =
      await run(deployAMMFactory, { version })
    const { TOKENS, EXCHANGES, LP_TOKENS } =
      await run(deployAMMExchanges, { FACTORY, ammVersion: version })
    console.log()
    console.info(bold('Deployed AMM contracts:'))
    printContracts([FACTORY,...EXCHANGES,...LP_TOKENS])
    console.log()
    return { FACTORY, TOKENS, EXCHANGES, LP_TOKENS }
  }
  /** Command. Take an existing AMM and create a new one with the same
    * contract templates. Recreate all the exchanges from the old exchange
    * in the new one. */
  static upgradeAMM = async function upgradeAMM ({
    run, chain, agent, deployment, prefix,
    oldVersion = 'v1',
    FACTORY = deployment.getThe(
      `AMM[${oldVersion}].Factory`, 
      new AMMFactoryContract({agent, version: oldVersion})
    ),
    newVersion = 'v2',
  }) {
    console.log()
    console.info(bold('Current factory:'))
    printContract(FACTORY)
    const EXCHANGES: ExchangeInfo[] = await FACTORY.exchanges
    await printExchanges(EXCHANGES)
    const { FACTORY: NEW_FACTORY } = await run(deployAMMFactory, { version: newVersion, copyFrom: FACTORY })
    printContract(NEW_FACTORY)
    const NEW_EXCHANGES = []
    if (!EXCHANGES) {
      console.warn('No exchanges in old factory.')
    } else {
      for (const { name, TOKEN_0, TOKEN_1 } of EXCHANGES) {
        console.log()
        console.info(bold('Creating V2 exchange'), name, 'from corresponding V1 exchange')
        NEW_EXCHANGES.push(saveExchange(
          { deployment, version: newVersion },
          await NEW_FACTORY.getContracts(),
          await NEW_FACTORY.createExchange(TOKEN_0, TOKEN_1)))
      }
      await printExchanges(NEW_EXCHANGES)
    }
    return { FACTORY: NEW_FACTORY, EXCHANGES: NEW_EXCHANGES }
  }
  /** Subclass. Sienna AMM Factory v1 */
  static v1 = class AMMFactoryContract_v1 extends AMMFactoryContract {
    version = 'v1'
    name    = `AMM[${this.version}].Factory`
    ref     = 'a99d8273b4'
    static deployAMM = function deployAMMFactory_v1 (input) {
      return AMMFactoryContract.deployAMM({ ...input, version: 'v1'})
    }
    static upgradeAMM = {
      to_v2: function upgradeAMMFactory_v1_to_v2 (input) {
        return AMMFactoryContract.upgradeAMM({...input, oldVersion:'v1', newVersion:'v2'})
      }
    }
  }
  /** Subclass. Sienna AMM Factory v2 */
  static v2 = class AMMFactoryContract_v2 extends AMMFactoryContract {
    version = 'v2'
    name    = `AMM[${this.version}].Factory`
    static deployAMM = async function deployAMMFactory_v2 (input) {
      return AMMFactoryContract.deployAMM({ ...input, version: 'v2'})
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
  /** Create a liquidity pool, i.e. an instance of the exchange contract,
    * and return info about it from getExchange. */
  async createExchange (
    token_0: SNIP20Contract|TokenType,
    token_1: SNIP20Contract|TokenType
  ): Promise<ExchangeInfo> {
    if (token_0 instanceof SNIP20Contract) token_0 = token_0.asCustomToken
    if (token_1 instanceof SNIP20Contract) token_1 = token_1.asCustomToken
    await this.tx().create_exchange(token_0, token_1)
    return await this.getExchange(token_0, token_1)
  }
  /** Get info about an exchange. */
  async getExchange (
    token_0: SNIP20Contract|TokenType,
    token_1: SNIP20Contract|TokenType
  ): Promise<ExchangeInfo> {
    if (token_0 instanceof SNIP20Contract) token_0 = token_0.asCustomToken
    if (token_1 instanceof SNIP20Contract) token_1 = token_1.asCustomToken
    const { agent, prefix, chain } = this
    const { address } = (await this.q(agent).get_exchange_address(token_0, token_1))
    const EXCHANGE = new AMMExchangeContract({
      chain,
      address,
      codeHash: await agent.getCodeHash(address),
      codeId:   await agent.getCodeId(address),
      prefix,
      agent,
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
      agent, prefix, chain,
      address:  liquidity_token.address,
      codeHash: liquidity_token.code_hash,
      codeId:   await agent.getCodeId(liquidity_token.address),
    })
    return {
      name,
      EXCHANGE, TOKEN_0, TOKEN_1, LP_TOKEN,
      raw: {
        exchange: { address: EXCHANGE.address },
        token_0,
        token_1,
        lp_token: { address: LP_TOKEN.address, code_hash: LP_TOKEN.codeHash },
      }
    }
  }
}

/** Deploy the Factory contract which is the hub of the AMM.
  * It needs to be passed code ids and code hashes for
  * the different kinds of contracts that it can instantiate.
  * So build and upload versions of those contracts too. */
export async function deployAMMFactory ({
  prefix, agent, chain, deployment,
  version = 'v2',
  copyFrom,
  initMsg = {
    admin:             agent.address,
    prng_seed:         randomHex(36),
    exchange_settings: getSettings(chain.id).amm.exchange_settings,
  }
}) {
  const options = { workspace, prefix, agent }
  const FACTORY   = new AMMFactoryContract[version]({ ...options })
  const LAUNCHPAD = new LaunchpadContract({ ...options })
  // launchpad is new to v2 so we build/upload it every time...
  await chain.buildAndUpload(agent, [FACTORY, LAUNCHPAD])
  const template = contract => ({ id: contract.codeId, code_hash: contract.codeHash })
  if (copyFrom) {
    const contracts = await copyFrom.getContracts()
    if (version === 'v2') {
      delete contracts.snip20_contract
      delete contracts.ido_contract
    }
    await deployment.init(agent, FACTORY, { ...initMsg, ...contracts })
  } else {
    const [EXCHANGE, AMMTOKEN, LPTOKEN, IDO] = await chain.buildAndUpload(agent, [
      new AMMExchangeContract({ ...options, version }),
      new AMMSNIP20Contract({   ...options }),
      new LPTokenContract({     ...options }),
      new IDOContract({         ...options }),
    ])
    const contracts = {
      snip20_contract:    template(AMMTOKEN),
      pair_contract:      template(EXCHANGE),
      lp_token_contract:  template(LPTOKEN),
      ido_contract:       template(IDO),
    }
    if (version === 'v2') {
      delete contracts.snip20_contract
      delete contracts.ido_contract
      delete contracts.launchpad_contract
    }
    await deployment.getOrInit(agent, FACTORY, 'SiennaAMMFactory', {
      ...initMsg,
      ...contracts
    })
  }
  console.info(
    bold(`Deployed factory ${version}`), FACTORY.label
  )
  printContract(FACTORY)
  return { FACTORY }
}

export async function deployAMMExchanges ({
  run, chain, agent, deployment,
  TOKENS    = { SIENNA: deployment.getThe('SIENNA', new SiennaSNIP20Contract({agent})) },
  FACTORY,
  EXCHANGES = [],
  LP_TOKENS = [],
  settings: { swapTokens, swapPairs } = getSettings(chain.id),
}) {
  // Collect referenced tokens, and created exchanges/LPs
  if (chain.isLocalnet) {
    // On localnet, deploy some placeholder tokens corresponding to the config.
    const { PLACEHOLDERS } = await run(deployPlaceholders)
    Object.assign(TOKENS, PLACEHOLDERS)
  } else {
    // On testnet and mainnet, talk to preexisting token contracts from the config.
    console.info(`Not running on localnet, using tokens from config:`)
    const tokens = {}
    // Make sure the correct code hash is being used for each token:
    for (
      let [name, {address, codeHash}] of
      Object.entries(swapTokens as Record<string, { address: string, codeHash: string }>)
    ) {
      // Soft code hash checking for now
      const realCodeHash = await agent.getCodeHash(address)
      if (codeHash !== realCodeHash) {
        console.warn(bold('Code hash mismatch for'), address, `(${name})`)
        console.warn(bold('  Config:'), codeHash)
        console.warn(bold('  Chain: '), realCodeHash)
      } else {
        console.info(bold(`Code hash of ${address}:`), realCodeHash)
      }
      // Always use real code hash - TODO bring settings up to date
      tokens[name] = new AMMSNIP20Contract({address, codeHash: realCodeHash, agent})
    }
    Object.assign(TOKENS, tokens)
  }
  // If there are any initial swap pairs defined in the config...
  for (const name of swapPairs) {
    // ...call the factory to deploy an EXCHANGE for each...
    const { EXCHANGE, LP_TOKEN } = await run(
      AMMExchangeContract.deploy, { FACTORY, TOKENS, name, version: FACTORY.version })
    // ...and collect the results
    EXCHANGES.push(EXCHANGE)
    LP_TOKENS.push(LP_TOKEN)
  }
  return { TOKENS, LP_TOKENS, EXCHANGES }
}

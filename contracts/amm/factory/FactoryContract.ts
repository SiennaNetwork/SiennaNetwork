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

import { FactoryClient } from './FactoryClient'
export class AMMFactoryContract extends Scrt_1_2.Contract<any, any> {

  workspace = workspace

  crate     = 'factory'

  client (agent: Agent): FactoryClient {
    return new FactoryClient(agent, this.address, this.codeHash)
  }

  /** Subclass. Sienna AMM Factory v1 */
  static v1 = class AMMFactoryContract_v1 extends AMMFactoryContract {
    version = 'v1'
    name    = `AMM[${this.version}].Factory`
    ref     = 'a99d8273b4'
    static deploy = function deployAMMFactory_v1 (input) {
      return AMMFactoryContract.deployImpl({ ...input, ammVersion: 'v1'})
    }
    static upgrade = {
      v2: function upgradeAMMFactory_v1_to_v2 (input) {
        return AMMFactoryContract.upgradeImpl({...input, oldVersion:'v1', newVersion:'v2'})
      }
    }
  }

  /** Subclass. Sienna AMM Factory v2 */
  static v2 = class AMMFactoryContract_v2 extends AMMFactoryContract {
    version = 'v2'
    name    = `AMM[${this.version}].Factory`
    static deploy = async function deployAMMFactory_v2 (input) {
      return AMMFactoryContract.deployImpl({ ...input, ammVersion: 'v2'})
    }
  }

  /** Command. Take the active TGE deployment, add the AMM Factory to it, use it to
    * create the configured AMM Exchange liquidity pools and their LP tokens. */
  protected static deployImpl = async function deployAMM ({
    run, suffix = `+${timestamp()}`,
    ammVersion
  }) {
    const { FACTORY } = await run(deployAMMFactory, { version: ammVersion, suffix })
    const { TOKENS, EXCHANGES, LP_TOKENS } = await run(deployAMMExchanges, { FACTORY, ammVersion })
    return {
      FACTORY,   // The deployed AMM Factory.
      TOKENS,    // Tokens supported by the AMM.
      EXCHANGES, // Exchanges that were created as part of the deployment
      LP_TOKENS  // LP tokens that were created as part of the deployment
    }
  }

  /** Command. Take an existing AMM and create a new one with the same
    * contract templates. Recreate all the exchanges from the old exchange
    * in the new one. */
  protected static upgradeImpl = async function upgradeAMM ({
    run, chain, agent, deployment, prefix,
    oldVersion = 'v1',
    newVersion = 'v2',
  }) {
    const name = `AMM[${oldVersion}].Factory`
    const FACTORY = deployment.getThe(name, new AMMFactoryContract({agent, name, version: oldVersion}))
    const EXCHANGES: ExchangeInfo[] = await FACTORY.client(agent).exchanges
    //await printExchanges(EXCHANGES)
    const { FACTORY: NEW_FACTORY } = await run(deployAMMFactory, { version: newVersion, copyFrom: FACTORY })
    const NEW_EXCHANGES = []
    if (!EXCHANGES) {
      console.warn('No exchanges in old factory.')
    } else {
      let newFactory = NEW_FACTORY.client(agent)
      for (const { name, TOKEN_0, TOKEN_1 } of EXCHANGES) {
        NEW_EXCHANGES.push(saveExchange(
          { deployment, ammVersion: newVersion },
          await newFactory.getContracts(),
          await newFactory.createExchange(TOKEN_0, TOKEN_1)
        ))
      }
    }
    return {
      FACTORY:   NEW_FACTORY,  // The AMM factory that was created as a result of the upgrade.
      EXCHANGES: NEW_EXCHANGES // The AMM exchanges that were created as a result of the upgrade.
    }
  }

}

/** Deploy the Factory contract which is the hub of the AMM.
  * It needs to be passed code ids and code hashes for
  * the different kinds of contracts that it can instantiate.
  * So build and upload versions of those contracts too. */
export async function deployAMMFactory ({
  prefix, agent, chain, deployment, suffix,
  version = 'v2',
  copyFrom,
  initMsg = {
    admin:             agent.address,
    prng_seed:         randomHex(36),
    exchange_settings: getSettings(chain.id).amm.exchange_settings,
  }
}) {
  const options = { prefix, agent }
  const FACTORY   = new AMMFactoryContract[version]({ ...options, suffix })
  const LAUNCHPAD = new LaunchpadContract({ ...options })
  // launchpad is new to v2 so we build/upload it every time...
  await chain.buildAndUpload(agent, [FACTORY, LAUNCHPAD])
  const template = contract => ({ id: contract.codeId, code_hash: contract.codeHash })
  if (copyFrom) {
    const contracts = await copyFrom.client(agent).getContracts()
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
  //console.info(
    //bold(`Deployed factory ${version}`), FACTORY.label
  //)
  //printContract(FACTORY)
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
    const { EXCHANGE, LP_TOKEN } = await run(AMMExchangeContract.deploy, {
      FACTORY, TOKENS, name, ammVersion: FACTORY.version
    })
    // ...and collect the results
    EXCHANGES.push(EXCHANGE)
    LP_TOKENS.push(LP_TOKEN)
  }
  return { TOKENS, LP_TOKENS, EXCHANGES }
}

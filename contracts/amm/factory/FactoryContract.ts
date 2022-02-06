import { Console, bold, colors, timestamp, randomHex } from '@hackbg/fadroma'

const console = Console('@sienna/amm/Factory')

import {
  Scrt_1_2, Snip20Contract, ContractInfo, Agent, MigrationContext, print
} from '@hackbg/fadroma'

import getSettings, { workspace } from '@sienna/settings'
import { AMMExchangeContract, ExchangeInfo, printExchanges } from '@sienna/exchange'
import { AMMSNIP20Contract } from '@sienna/amm-snip20'
import { LPTokenContract } from '@sienna/lp-token'
import { IDOContract } from '@sienna/ido'
import { LaunchpadContract } from '@sienna/launchpad'
import { SiennaSnip20Contract } from '@sienna/snip20-sienna'

import { InitMsg, ExchangeSettings, ContractInstantiationInfo } from './schema/init_msg.d'
import { TokenType } from './schema/handle_msg.d'
import { QueryResponse, Exchange } from './schema/query_response.d'
import { FactoryClient } from './FactoryClient'
export { FactoryClient }

export abstract class AMMFactoryContract extends Scrt_1_2.Contract<FactoryClient> {

  abstract name: string

  source = { workspace, crate: 'factory' }

  Client = FactoryClient

  /** Subclass. Sienna AMM Factory v1 */
  static v1 = class AMMFactoryContract_v1 extends AMMFactoryContract {
    version = 'v1'
    name    = `AMM[${this.version}].Factory`
    source  = { workspace, crate: 'factory', ref: '2f75175212'/*'a99d8273b4'???*/ }
    static deploy = function deployAMMFactory_v1 (input) {
      return AMMFactoryContract.deployAMM({ ...input, ammVersion: 'v1'})
    }
    static upgrade = {
      v2: function upgradeAMMFactory_v1_to_v2 (input) {
        return AMMFactoryContract.upgradeAMM({...input, oldVersion: 'v1', newVersion: 'v2'})
      }
    }
  }

  /** Subclass. Sienna AMM Factory v2 */
  static v2 = class AMMFactoryContract_v2 extends AMMFactoryContract {
    version = 'v2'
    name    = `AMM[${this.version}].Factory`
    static deploy = async function deployAMMFactory_v2 (input) {
      return AMMFactoryContract.deployAMM({ ...input, ammVersion: 'v2'})
    }
  }

  /** Command. Take the active TGE deployment, add the AMM Factory to it, use it to
    * create the configured AMM Exchange liquidity pools and their LP tokens. */
  protected static deployAMM = deployAMM

  /** Command. Take an existing AMM and create a new one with the same
    * contract templates. Recreate all the exchanges from the old exchange
    * in the new one. */
  protected static upgradeAMM = upgradeAMM

}

async function deployAMM ({
  run, suffix = `+${timestamp()}`,
  ammVersion
}) {
  const { FACTORY } = await run(deployAMMFactory, {
    version: ammVersion,
    suffix
  })
  const { EXCHANGES, LP_TOKENS } = await run(AMMExchangeContract.deployMany, {
    FACTORY,
    ammVersion
  })
  return {
    FACTORY,   // The deployed AMM Factory.
    EXCHANGES, // Exchanges that were created as part of the deployment
    LP_TOKENS  // LP tokens that were created as part of the deployment
  }
}

async function upgradeAMM ({
  run, chain, agent, deployment, prefix, suffix = `+${timestamp()}`,
  oldVersion = 'v1',
  newVersion = 'v2',
}) {

  const name = `AMM[${oldVersion}].Factory`
  const FACTORY = deployment.getThe(
    name,
    new AMMFactoryContract[oldVersion]({ agent, name, version: oldVersion })
  )

  const EXCHANGES = await FACTORY.client(agent).exchanges

  const { FACTORY: NEW_FACTORY } = await run(deployAMMFactory, {
    version: newVersion, copyFrom: FACTORY.client(agent), suffix
  })

  const NEW_EXCHANGES = []

  if (!EXCHANGES || EXCHANGES.length === 0) console.warn('No exchanges in old factory.')

  const factory = NEW_FACTORY.client(agent)
  const inventory = await factory.getContracts()
  const ammVersion = newVersion
  await agent.bundle(async agent=>{
    const factory = NEW_FACTORY.client(agent)
    for (const { name, TOKEN_0, TOKEN_1 } of (EXCHANGES||[])) {
      const exchange = await factory.createExchange(TOKEN_0, TOKEN_1)
      NEW_EXCHANGES.push([TOKEN_0, TOKEN_1])
    }
  })

  return {

    // The AMM factory that was created as a result of the upgrade.
    FACTORY: NEW_FACTORY,

    // The AMM exchanges that were created as a result of the upgrade.
    EXCHANGES: await Promise.all(NEW_EXCHANGES.map(async ([TOKEN_0, TOKEN_1])=>{
      const exchange = await factory.getExchange(TOKEN_0, TOKEN_1)
      return AMMExchangeContract.save({ deployment, ammVersion, inventory, exchange })
    }))

  }

}

/** Deploy the Factory contract which is the hub of the AMM.
  * It needs to be passed code ids and code hashes for
  * the different kinds of contracts that it can instantiate.
  * So build and upload versions of those contracts too. */
export async function deployAMMFactory (context) {
  const {
    agent, deployment, prefix, suffix = timestamp(),
    version = 'v2',
    copyFrom,
    initMsg = {
      admin:             agent.address,
      prng_seed:         randomHex(36),
      exchange_settings: getSettings(agent.chain.id).amm.exchange_settings,
    },
  } = context
  const FACTORY = new AMMFactoryContract[version]({ prefix, suffix })
  await agent.chain.buildAndUpload(agent, [FACTORY])
  const templates = copyFrom
    ? await copyFrom.client(agent).getContracts()
    : await buildTemplates(agent, version)
  if (version === 'v2') {
    delete templates.snip20_contract
    delete templates.ido_contract
    delete templates.launchpad_contract
  }
  const factoryInitMsg = { ...initMsg, ...templates }
  await deployment.instantiate(agent, [FACTORY, factoryInitMsg])
  return { FACTORY: FACTORY.client(agent) }
}

async function buildTemplates (agent: Agent, version: 'v1'|'v2') {
  console.log({version}, AMMExchangeContract.v1)
  const AMMTOKEN  = new AMMSNIP20Contract()
  const LPTOKEN   = new LPTokenContract()
  const EXCHANGE  = new AMMExchangeContract[version]()
  const LAUNCHPAD = new LaunchpadContract() // special cased because versions
  const IDO       = new IDOContract()
  await agent.chain.buildAndUpload(agent, [AMMTOKEN, LPTOKEN, EXCHANGE, LAUNCHPAD, IDO])
  const template = contract => ({
    id:        Number(contract.template.codeId),
    code_hash: contract.template.codeHash
  })
  return {
    snip20_contract:    template(AMMTOKEN),
    pair_contract:      template(EXCHANGE),
    lp_token_contract:  template(LPTOKEN),
    ido_contract:       template(IDO),
    launchpad_contract: template(LAUNCHPAD),
  }
}

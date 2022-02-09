import { Console, bold, colors, timestamp, randomHex, Template, ScrtBundle } from '@hackbg/fadroma'

const console = Console('@sienna/amm/Factory')

import {
  Scrt_1_2, Snip20Contract, ContractInfo, Agent, MigrationContext, print
} from '@hackbg/fadroma'

import getSettings, { workspace } from '@sienna/settings'
import { AMMExchangeContract, AMMExchangeClient, ExchangeInfo, printExchanges } from '@sienna/exchange'
import { AMMSNIP20Contract } from '@sienna/amm-snip20'
import { LPTokenContract, LPTokenClient } from '@sienna/lp-token'
import { IDOContract } from '@sienna/ido'
import { LaunchpadContract } from '@sienna/launchpad'
import { SiennaSnip20Contract } from '@sienna/snip20-sienna'

import { InitMsg, ExchangeSettings, ContractInstantiationInfo } from './schema/init_msg.d'
import { TokenType } from './schema/handle_msg.d'
import { QueryResponse, Exchange } from './schema/query_response.d'
import { AMMVersion, AMMFactoryClient, AMMFactoryTemplates } from './FactoryClient'
export { AMMFactoryClient }

export abstract class AMMFactoryContract extends Scrt_1_2.Contract<AMMFactoryClient> {

  abstract name: string
  abstract version: AMMVersion

  /** Subclass. Sienna AMM Factory v1 */
  static v1 = class AMMFactoryContract_v1 extends AMMFactoryContract {
    version = 'v1' as AMMVersion
    name    = `AMM[${this.version}].Factory`
    source  = { workspace, crate: 'factory', ref: '2f75175212'/*'a99d8273b4'???*/ }
    Client  = AMMFactoryClient[this.version]
    static deploy = function deployAMMFactory_v1 (input) {
      return deployAMM({ ...input, ammVersion: 'v1'})
    }
    static upgrade = {
      v2: function upgradeAMMFactory_v1_to_v2 (input) {
        return upgradeAMM({...input, oldVersion: 'v1', newVersion: 'v2'})
      }
    }
  }

  /** Subclass. Sienna AMM Factory v2 */
  static v2 = class AMMFactoryContract_v2 extends AMMFactoryContract {
    version = 'v2' as AMMVersion
    name    = `AMM[${this.version}].Factory`
    source  = { workspace, crate: 'factory', ref: 'HEAD' }
    Client  = AMMFactoryClient[this.version]
    static deploy = async function deployAMMFactory_v2 (input) {
      return deployAMM({ ...input, ammVersion: 'v2'})
    }
  }

}

/** Command. Take the active TGE deployment, add the AMM Factory to it, use it to
  * create the configured AMM Exchange liquidity pools and their LP tokens. */
async function deployAMM ({
  agent, run, suffix = `+${timestamp()}`,
  ammVersion
}: MigrationContext & {
  ammVersion: string
}): Promise<{
  FACTORY:   AMMFactoryClient,
  EXCHANGES: AMMExchangeClient[],
  LP_TOKENS: LPTokenClient[]
}> {
  const [template] = await agent.buildAndUpload([new AMMFactoryContract[ammVersion]()])
  const FACTORY = await run(deployAMMFactory, { template, version: ammVersion, suffix })
  const exchangeOptions = { FACTORY, ammVersion }
  const { EXCHANGES, LP_TOKENS } = await run(AMMExchangeContract.deployMany, exchangeOptions)
  return {
    FACTORY,   // The deployed AMM Factory.
    EXCHANGES, // Exchanges that were created as part of the deployment
    LP_TOKENS  // LP tokens that were created as part of the deployment
  }
}

/** Command. Take an existing AMM and create a new one with the same
  * contract templates. Recreate all the exchanges from the old exchange
  * in the new one. */
async function upgradeAMM (context: MigrationContext & {

  generateMigration: boolean

  oldVersion:     AMMVersion
  oldFactoryName: string
  oldFactory:     AMMFactoryClient
  oldExchanges:   AMMExchangeContract[]
  oldTemplates:   any,

  newVersion:     AMMVersion,

  name: string,

}): Promise<ScrtBundle|{
  FACTORY:   AMMFactoryClient
  EXCHANGES: ExchangeInfo[]
}> {
  const {
    generateMigration = false,
    run, chain, agent, deployment, prefix, suffix = `+${timestamp()}`,

    // auto-get the old factory and its exchanges by default;
    // still allow them to be passed in for multisig mode
    oldVersion     = 'v1',
    oldFactoryName = `AMM[${oldVersion}].Factory`,
    oldFactory     = new AMMFactoryClient[oldVersion]({ ...deployment.get(oldFactoryName), agent }),
    oldExchanges   = await oldFactory.listExchangesFull(),
    oldTemplates   = await oldFactory.getContracts(),

    newVersion = 'v2',
  } = context

  // upload the new factory's code
  const [newFactoryTemplate] = await agent.buildAndUpload([
    new AMMFactoryContract[newVersion]({ prefix, suffix })
  ])

  // if we're generating the multisig transactions,
  // skip the queries and store all the txs in a bundle
  let bundle
  if (generateMigration) bundle = agent.bundle()

  // create the new factory instance
  const newFactory = await run(deployAMMFactory, {
    agent:     generateMigration ? bundle : agent,
    version:   newVersion,
    template:  newFactoryTemplate,
    templates: oldTemplates,
    suffix
  }) as AMMFactoryClient

  // create the new exchanges, collecting the pair tokens
  const newPairs = await newFactory.createExchanges({
    pairs:     oldExchanges,
    templates: oldTemplates
  })

  if (!generateMigration) {
    // turn the list of pairs to create
    // into a list of created exchange instances
    const newExchanges = await Promise.all(newPairs.map(
      ({TOKEN_0, TOKEN_1})=>newFactory.getExchange(
        TOKEN_0.asCustomToken,
        TOKEN_1.asCustomToken
      )
    ))
    const inventory  = await newFactory.getContracts()
    const ammVersion = newVersion
    // save the newly created contracts to the deployment
    newExchanges.forEach((exchange)=>AMMExchangeContract.save({
      deployment, ammVersion, inventory, exchange
    }))
  }

  return generateMigration ? bundle : {
    // The AMM factory that was created as a result of the upgrade.
    FACTORY: newFactory,
    // The AMM exchanges that were created as a result of the upgrade.
    EXCHANGES: newExchanges
  }
}

/** Deploy the Factory contract which is the hub of the AMM.
  * It needs to be passed code ids and code hashes for
  * the different kinds of contracts that it can instantiate.
  * So build and upload versions of those contracts too. */
export async function deployAMMFactory (context: MigrationContext & {
  /** Version of the factory that will be deployed. */
  version:    AMMVersion,
  /** Code id and code hash for the Factory contract that will be deployed. */
  template:   Template,
  /** Code ids and code hashes of the contracts that the factory can spawn. */
  templates?: AMMFactoryTemplates,
  /** Configuration of the factory - goes into initMsg */
  config:     any
}): Promise<AMMFactoryClient> {
  const {
    agent, deployment, prefix, suffix = timestamp(),
    version   = 'v2',
    template,
    templates = await buildTemplates(agent, version),
    config    = {
      admin:             agent.address,
      prng_seed:         randomHex(36),
      exchange_settings: getSettings(agent.chain.id).amm.exchange_settings,
    },
  } = context
  if (version === 'v2') {
    delete templates.snip20_contract
    delete templates.ido_contract
    delete templates.launchpad_contract
  }
  const factory = new AMMFactoryContract[version]({ prefix, suffix })
  factory.template = template
  await deployment.instantiate(agent, [factory, {...config, ...templates}])
  return factory.client(agent)
}

async function buildTemplates (agent: Agent, version: 'v1'|'v2') {
  const AMMTOKEN  = new AMMSNIP20Contract()
  const LPTOKEN   = new LPTokenContract()
  const EXCHANGE  = new AMMExchangeContract[version]()
  const LAUNCHPAD = new LaunchpadContract() // special cased because versions
  const IDO       = new IDOContract()
  for (const contract of [AMMTOKEN, LPTOKEN, EXCHANGE, LAUNCHPAD, IDO]) {
    await agent.buildAndUpload([contract]) // TODO parallel
  }
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

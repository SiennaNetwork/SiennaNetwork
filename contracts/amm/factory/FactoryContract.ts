import { Console, bold, colors, timestamp, randomHex } from '@hackbg/fadroma'

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
import { AMMVersion, AMMFactoryClient } from './FactoryClient'
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
  run, suffix = `+${timestamp()}`,
  ammVersion
}: MigrationContext & {
  ammVersion: string
}): Promise<{
  FACTORY:   AMMFactoryClient,
  EXCHANGES: AMMExchangeClient[],
  LP_TOKENS: LPTokenClient[]
}> {
  const factoryOptions = { version: ammVersion, suffix }
  const { FACTORY } = await run(deployAMMFactory, factoryOptions)
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
async function upgradeAMM ({
  run, chain, agent, deployment, prefix, suffix = `+${timestamp()}`,
  oldVersion = 'v1',
  newVersion = 'v2',
}): Promise<{
  FACTORY:   AMMFactoryClient
  EXCHANGES: ExchangeInfo[]
}> {
  // get the old factory and its exchanges
  const name         = `AMM[${oldVersion}].Factory`
  const oldFactory   = new AMMFactoryClient[oldVersion]({ ...deployment.get(name), agent })
  const oldExchanges = await oldFactory.listExchangesFull()

  // create the new factory
  const newFactoryOptions = { version: newVersion, copyFrom: oldFactory, suffix }
  const { FACTORY: newFactory } = await run(deployAMMFactory, newFactoryOptions)

  // create the new exchanges, collecting the pair tokens
  const newExchanges = await newFactory.createExchanges(oldExchanges)
  const inventory  = await newFactory.getContracts()
  const ammVersion = newVersion
  for (const exchange of newExchanges) {
    AMMExchangeContract.save({ deployment, ammVersion, inventory, exchange })
  }

  return {
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
export async function deployAMMFactory ({
  agent, deployment, prefix, suffix = timestamp(),
  version = 'v2',
  copyFrom,
  initMsg = {
    admin:             agent.address,
    prng_seed:         randomHex(36),
    exchange_settings: getSettings(agent.chain.id).amm.exchange_settings,
  },
}: MigrationContext & {
  version:   AMMVersion,
  copyFrom?: AMMFactoryClient,
  initMsg:   any
}): Promise<{
  FACTORY: AMMFactoryClient
}> {
  const FACTORY = new AMMFactoryContract[version]({ prefix, suffix })
  await agent.buildAndUpload([FACTORY])
  const templates = copyFrom
    ? await copyFrom.getContracts()
    : await buildTemplates(agent, version)
  if (version === 'v2') {
    delete templates.snip20_contract
    delete templates.ido_contract
    delete templates.launchpad_contract
  }
  const factoryInitMsg = { ...initMsg, ...templates }
  await deployment.instantiate(agent, [FACTORY, factoryInitMsg])
  return {
    FACTORY: FACTORY.client(agent)
  }
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

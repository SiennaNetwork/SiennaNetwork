import { MigrationContext, Template, bold } from '@hackbg/fadroma'
import * as API from '@sienna/api'
import { buildAMMTemplates } from '../Build'
import { uploadAMMTemplates } from '../Upload'
import * as Tokens from '../Tokens'

export interface AMMDeployOptions {
  /** The version of the AMM to deploy */
  ammVersion: API.AMMVersion
}

export interface AMMDeployResult {
  /** The deployed AMM Factory */
  FACTORY:   API.AMMFactoryClient,
  /** The exchanges that were created */
  EXCHANGES: API.AMMExchangeClient[],
  /** The LP tokens that were created */
  LP_TOKENS: API.LPTokenClient[]
}

export interface AMMFactoryDeployOptions {
  /** Version of the factory to deploy. */
  version:    API.AMMVersion,
  /** Code id and hash for the factory to deploy */
  template:   Template,
  /** Relevant properties from global project config. */
  settings: { amm: { exchange_settings: object } }
  /** Config of new factory - goes into initMsg */
  config: {
    admin:             string,
    prng_seed:         string,
    exchange_settings: object
  }
  /** Code ids+hashes of contracts
    * that the new factory can instantiate. */
  templates?: AMMFactoryTemplates,
}

export interface AMMExchangesDeployOptions {
  settings: { swapPairs: string[] }
  knownTokens: any,
  FACTORY:     API.AMMFactoryClient,
  ammVersion:  API.AMMVersion
}

export interface AMMUpgradeOptions {
  builder:            Builder
  generateMigration:  boolean
  vOld:               API.AMMVersion
  oldFactoryName:     string
  oldFactory:         API.AMMFactoryClient
  oldExchanges:       API.AMMExchangeClient[]
  oldTemplates:       any,
  vNew:               API.AMMVersion,
  newRef:             string,
  newFactoryTemplate: Template
  name: string,
}

export type AMMUpgradeResult = ScrtBundle | {
  // The factory that was created by the upgrade.
  FACTORY:   API.AMMFactoryClient
  // The exchanges that were created by the upgrade.
  EXCHANGES: API.ExchangeInfo[]
  // what about the LP tokens?
}

export interface RedeployAMMExchangeOptions {
  NEW_FACTORY:   unknown,
  OLD_EXCHANGES: unknown,
  ammVersion:    AMMVersion
}

export interface RedeployAMMExchangeResult {
  NEW_EXCHANGES: unknown
}

export async function deployAMM (
  context: MigrationContext & AMMDeployOptions
): Promise<AMMDeployResult> {
  const { run, ammVersion, ref } = context
  console.info('deployAMM', { ref })
  const FACTORY =
    await run(deployAMMFactory, { version: ammVersion, ref })
  const { EXCHANGES, LP_TOKENS } =
    await run(deployAMMExchanges, { FACTORY, ammVersion, ref })
  return { FACTORY, EXCHANGES, LP_TOKENS }
}

export async function deployAMMFactory (
  context: MigrationContext & AMMFactoryDeployOptions
): Promise<AMMFactoryClient> {
  // Default settings:
  const {
    version   = 'v2',
    ref       = versions.AMM[version],
    src       = source('factory', ref),

    builder,
    artifact  = await builder.build(source('factory', ref)),

    uploader,
    template  = await uploader.upload(artifact),
    templates = await buildAMMTemplates(uploader, version, ref),

    deployAgent, deployment, prefix,

    agent, 
    settings: { amm: { exchange_settings } } = getSettings(agent.chain.mode),
    config = {
      admin: agent.address,
      prng_seed: randomHex(36),
      exchange_settings
    }
  } = context
  console.info('deployAMMFactory', { ref })
  // If the templates are copied from v1, remove the extra templates
  if (version !== 'v1') {
    delete templates.snip20_contract
    delete templates.ido_contract
    delete templates.launchpad_contract
  }
  // Instantiate the new factory and return a client to it
  const name     = `AMM[${version}].Factory`
  const initMsg  = { ...config, ...templates }
  const instance = await deployment.init(
    deployAgent, template, name, initMsg
  )
  return new API.AMMFactoryClient[version]({
    ...deployment.get(name), agent
  })
}

export async function deployAMMExchanges (options: MigrationContext & AMMExchangesDeployOptions) {
  const {
    run, agent, deployment,
    settings: { swapPairs } = getSettings(agent.chain.mode),
    knownTokens = await run(Tokens.getSupported),
    FACTORY,
    ammVersion
  } = options
  if (swapPairs.length > 0) {
    const createdPairs = []
    await agent.bundle().wrap(async bundle=>{
      const agent = FACTORY.agent
      FACTORY.agent = bundle
      const factory = new API.AMMFactoryClient({...FACTORY})
      for (const name of swapPairs) {
        const { token0, token1 } = Tokens.fromPairName(knownTokens, name)
        await factory.createExchange(token0, token1)
        createdPairs.push([token0, token1])
      }
      FACTORY.agent = agent
    })
    const { EXCHANGES } = await run(Receipts.saveCreatedPairs, {
      FACTORY, ammVersion, createdPairs
    })
    return {
      EXCHANGES: EXCHANGES.map(EXCHANGE=>EXCHANGE.EXCHANGE),
      LP_TOKENS: EXCHANGES.map(EXCHANGE=>EXCHANGE.LP_TOKEN)
    }
  }
}

/** This procedure deploys a new exchange.
  * If the exchange already exists, it does nothing.
  * Factory doesn't allow 2 identical exchanges to exist anyway,
  * as compared by `TOKEN0` and `TOKEN1`. */
async function deployAMMExchange (options) {
  const {
    agent, deployment, run,
    knownTokens = await run(Tokens.getSupportedTokens),
    FACTORY,
    name,
    ammVersion
  } = options
  const factory   = FACTORY.client(agent)
  const inventory = await factory.getTemplates()
  const { token0, token1 } = Tokens.fromName(knownTokens, name)
  try {
    const { EXCHANGE, LP_TOKEN } =
      await factory.getExchange(token0, token1)
    EXCHANGE.prefix = LP_TOKEN.prefix = deployment.prefix
    console.info(`${bold(name)}: Already exists.`)
    return { EXCHANGE, LP_TOKEN }
  } catch (e) {
    if (e.message.includes("Address doesn't exist in storage")) {
      await factory.createExchange(token0, token1)
      const exchange = await factory.getExchange(token0, token1)
      return Receipts.saveAMMExchange({
        deployment, ammVersion, inventory, exchange
      })
    } else {
      console.error(e)
      throw new Error(
        `${bold(`Factory::GetExchange(${name})`)}: '+
        'not found (${e.message})`
      )
    }
  }
}

export async function deployRouter (
  context: MigrationContext
): Promise {

  const { builder
        , uploader
        , ref = versions.HEAD
        , template = await buildAndUpload(builder, uploader, source('router', ref))
        , deployAgent, deployment, prefix
        , agent
        } = context

  // Define name for deployed contracts
  const v = 'v2'
  const name = `AMM[${v}].Router`

  // Deploy router
  const router = await deployment.init(deployAgent, template, name, {})

  // Return clients to the instantiated contracts
  return { router }
}

export async function upgradeAMM (
  context: MigrationContext & AMMUpgradeOptions
): Promise<AMMUpgradeResult> {

  const {
    run,

    builder,
    uploader,

    deployment, prefix,
    agent, chain,

    generateMigration = false,

    // By default, the old factory and its exchanges
    // are automatically retrieved; context still allows
    // them to be passed in manually (for multisig mode?)
    vOld = 'v1',
    oldFactoryName = `AMM[${vOld}].Factory`,
    oldFactory     = new API.AMMFactoryClient[vOld]({
      ...deployment.get(oldFactoryName), agent
    }),
    oldExchanges = await oldFactory.listExchangesFull(),
    oldTemplates = await oldFactory.getTemplates(),

    vNew = 'v2',
    newRef = versions.AMM[vNew],
    newFactoryTemplate = await buildAndUpload(builder, uploader, source('factory', ref))
  } = context

  // if we're generating the multisig transactions,
  // skip the queries and store all the txs in a bundle
  let bundle
  if (generateMigration) bundle = agent.bundle()

  // create the new factory instance
  const newFactory = await run(deployAMMFactory, {
    agent:     generateMigration ? bundle : agent,
    version:   vNew,
    template:  newFactoryTemplate,
    templates: oldTemplates,
  }) as API.AMMFactoryClient

  // create the new exchanges, collecting the pair tokens
  const newPairs = await newFactory.createExchanges({
    pairs:     oldExchanges,
    templates: oldTemplates
  })

  let newExchanges
  if (!generateMigration) {
    console.log(newPairs.sort())
    newExchanges = await Receipts.saveExchangeReceipts(
      deployment, vNew, newFactory, newPairs
    )
  }

  return generateMigration ? bundle : {
    FACTORY:   newFactory,
    EXCHANGES: newExchanges
  }

}

export async function upgradeAMMFactory_v1_to_v2 (context) {
  const {
    run, deployment, prefix, clientAgent
  } = context
  const v1: Record<string, any> = {}
  v1.name = `AMM[v1].Factory`
  v1.factory = new API.AMMFactoryClient.v1({ ...deployment.get(v1.name), agent: clientAgent })
  const v2: Record<string, any> = {}
  v2.client  = await run(deployAMMFactory, { version: 'v2' })
  return { v1, v2 }
}

export async function cloneAMMExchanges_v1_to_v2 (context) {
  const { run, deployment, clientAgent, deployAgent } = context
  const v1: Record<string, any> = {}
  v1.name    = `AMM[v1].Factory`
  v1.factory = new API.AMMFactoryClient.v1({
    ...deployment.get(v1.name), agent: clientAgent
  })
  v1.pairs   = await v1.factory.listExchanges()
  console.info(bold(`AMM v1:`), v1.pairs.length, 'pairs')
  const v2: Record<string, any> = {}
  v2.name      = `AMM[v2].Factory`
  v2.readFactory  = new API.AMMFactoryClient.v2({
    ...deployment.get(v2.name), agent: clientAgent
  })
  v2.templates = await v2.readFactory.getTemplates()
  v2.existing  = await v2.readFactory.listExchanges()
  const existingV1PairsJSON = v1.pairs.map(x=>JSON.stringify(x.pair))
  const existingV2PairsJSON = v2.existing.map(x=>JSON.stringify(x.pair))
  const v2PairsToCreate = []
  for (const v1pairJSON of existingV1PairsJSON) {
    if (existingV2PairsJSON.includes(v1pairJSON)) {
      console.warn(bold(`Pair exists, not creating:`), v1pairJSON)
    } else {
      console.info(bold(`Will create pair:`), v1pairJSON)
      v2PairsToCreate.push({ pair: JSON.parse(v1pairJSON) })
    }
  }
  v2.writeFactory = new API.AMMFactoryClient.v2({
    ...deployment.get(v2.name), agent: deployAgent
  })
  console.log({read: v2.readFactory, write: v2.writeFactory})
  v2.pairs = await v2.writeFactory.createExchanges({
    templates: v2.templates,
    pairs:     v2PairsToCreate
  })
  v2.exchanges = await Receipts.saveExchangeReceipts(
    deployment, 'v2', v2.readFactory, v2.pairs
  )
  return { v1, v2 }
}

export async function redeployAMMExchanges (
  context: MigrationContext & RedeployAMMExchangeOptions
): Promise<RedeployAMMExchangeResult> {
  const {
    agent, deployment,
    ammVersion, NEW_FACTORY, OLD_EXCHANGES = [],
  } = context
  // 1. create them in one go
  let NEW_EXCHANGES = []
  await agent.bundle(async agent=>{
    const bundled = NEW_FACTORY.client(agent)
    for (const { name, TOKEN_0, TOKEN_1 } of (OLD_EXCHANGES||[])) {
      const exchange = await bundled.createExchange(TOKEN_0, TOKEN_1)
      NEW_EXCHANGES.push([TOKEN_0, TOKEN_1])
    }
  })
  // 2. get them
  const factory = NEW_FACTORY.client(agent)
  const inventory = await NEW_FACTORY.client(agent).getTemplates()
  // 3. save the receipts
  const save = async ([TOKEN_0, TOKEN_1])=>{
    const exchange = await factory.getExchange(TOKEN_0, TOKEN_1)
    return Receipts.saveAMMExchange({
      deployment, ammVersion, inventory, exchange
    })
  }
  return { NEW_EXCHANGES: await Promise.all(NEW_EXCHANGES.map(save)) }
}

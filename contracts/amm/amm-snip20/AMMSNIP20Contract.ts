import { Agent, MigrationContext, randomHex, bold, timestamp, Console } from '@hackbg/fadroma'
import { Snip20Contract, Snip20Contract_1_2 } from '@hackbg/fadroma'
import getSettings, { workspace } from '@sienna/settings'
import { InitMsg } from './schema/init_msg.d'

const console = Console('@sienna/amm-snip20')

export class AMMSNIP20Contract extends Snip20Contract_1_2 {

  name  = 'AMM.SNIP20'

  source = { workspace, crate: 'amm-snip20' }

  initMsg: InitMsg = {
    prng_seed: randomHex(36),
    name:      "",
    symbol:    "",
    decimals:  18,
    config:    {
      public_total_supply: true,
      enable_mint:         true
    },
  }

  /** Procedure. Convert token name to token descriptor. */
  static tokenFromName        = tokensFromName
  /** Procedure. Convert pair in format TOKEN0-TOKEN1 to token pair descriptor. */
  static tokensFromName       = tokensFromName
  /** Procedure. Get a collection of all supported tokens, including placeholders if localnet. */
  static getSupportedTokens   = getSupportedTokens
  /** Procedure. Get a collection of the placeholder tokens used on localnet. */
  static getPlaceholderTokens = getPlaceholderTokens

}

async function tokenFromName (options) {
  const {
    run,
    TOKENS = (await run(AMMSNIP20Contract.getSupportedTokens)).TOKENS,
    name   = 'UNKNOWN'
  } = options
  if (!TOKENS[name]) throw new Error(
    `Missing token ${name}; available: ${Object.keys(TOKENS).join(' ')}`
  )
  return {
    token: TOKENS[name].asCustomToken
  }
}

async function tokensFromName (options) {
  const {
    run,
    TOKENS = (await run(AMMSNIP20Contract.getSupportedTokens)).TOKENS,
    name   = 'UNKNOWN-UNKNOWN'
  } = options
  const [name0, name1] = name.split('-')
  return {
    token0: (await run(tokenFromName, { TOKENS, name: name0 })).token,
    token1: (await run(tokenFromName, { TOKENS, name: name1 })).token
  }
}

async function getSupportedTokens (options) {
  const {
    agent, deployment, run,
    settings: { swapTokens } = getSettings(agent.chain.id),
    TOKENS = {
      // Support own token
      SIENNA: deployment.getThe('SIENNA', new Snip20Contract({agent}))
    }
  } = options

  // On localnet, support placeholder tokens
  if (agent.chain.isLocalnet) {
    // On localnet, deploy some placeholder tokens corresponding to the config.
    await run(getPlaceholderTokens, { TOKENS })
  }

  // On testnet and mainnet, use preexisting token contracts specified in the config.
  if (!agent.chain.isLocalnet) {
    await run(getConfiguredTokens, { TOKENS })
  }

  return { TOKENS }
}

async function getConfiguredTokens (options) {

  console.info(`Not running on localnet, using tokens from config:`)

  const {
    agent,
    settings = getSettings(agent.chain.id),
    TOKENS = {}
  } = options

  const swapTokens: Record<string, { address: string, codeHash: string }> = settings.swapTokens

  await Promise.all(
    Object.entries(swapTokens).map(
      async ([name, {address, codeHash}])=>{
        TOKENS[name] = new AMMSNIP20Contract({
          address,
          codeHash: await agent.checkCodeHash(address, codeHash),
          agent
        })
      }
    )
  )

  return { TOKENS }

}

async function getPlaceholderTokens (options) {

  const {
    agent, deployment, prefix,
    settings = getSettings(agent.chain.id),
    TOKENS = {}
  } = options

  const toDeploy = []

  const placeholders: Record<string, { label: string, initMsg: any }> = settings.placeholderTokens
  for (const [_, {label, initMsg}] of Object.entries(placeholders)) {
    const { symbol } = initMsg
    if (TOKENS[symbol]) {
      console.info(bold(symbol), 'exists in working memory')
      continue
    }
    const name = `Placeholder.${label}`
    if (deployment.receipts[name]) {
      console.info(bold(name), 'exists in current deployment')
      TOKENS[symbol] = deployment.getThe(name, new AMMSNIP20Contract({}))
      continue
    }
    console.info(bold(name), 'deploying...')
    const TOKEN = TOKENS[symbol] = new AMMSNIP20Contract({ prefix, name })
    await agent.chain.buildAndUpload(agent, [TOKEN])
    toDeploy.push([symbol, TOKEN, { ...initMsg, name: initMsg.symbol }])
  }

  if (toDeploy.length > 0) {
    // deploy all placeholders in 1 tx
    console.log(1)
    await agent.bundle(async agent=>{
      for (const [_, TOKEN, initMsg] of toDeploy) {
        await deployment.init(agent, TOKEN, initMsg)
      }
    })
    console.log(2)
    // mint test balances for all placeholders in 1 tx
    await agent.bundle(async agent=>{
      for (const [symbol, _, __] of toDeploy) {
        const tokenTX = TOKENS[symbol].tx(agent)
        const amount = "100000000000000000000000"
        console.warn("Minting", bold(amount), bold(symbol), 'to', bold(agent.address))
        const {address} = agent
        await tokenTX.setMinters([address])
        await tokenTX.mint(amount, address)
      }
    })
  }

  return { TOKENS }

}

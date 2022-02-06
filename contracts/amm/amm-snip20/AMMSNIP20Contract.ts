import { Agent, MigrationContext, randomHex, bold, timestamp, Console } from '@hackbg/fadroma'
import { Snip20Contract, Snip20Contract_1_2 } from '@hackbg/fadroma'
import getSettings, { workspace } from '@sienna/settings'
import { InitMsg } from './schema/init_msg.d'

const console = Console('@sienna/amm-snip20')

import { AMMSNIP20Client } from './AMMSNIP20Client'
export { AMMSNIP20Client }
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

  /** Convert token name to token descriptor. */
  static tokenFromName        = tokensFromName
  /** Convert pair in format TOKEN0-TOKEN1 to token pair descriptor. */
  static tokensFromName       = tokensFromName
  /** Get a collection of all supported tokens, including placeholders if localnet. */
  static getSupportedTokens   = getSupportedTokens
  /** Get handles to tokens from swapTokens. */
  static getConfiguredTokens  = getConfiguredTokens
  /** Get a collection of the placeholder tokens used on localnet. */
  static getPlaceholderTokens = getPlaceholderTokens

}

async function tokenFromName ({
  run,
  TOKENS, name = 'UNKNOWN'
}): Promise<TokenType> {
  if (!TOKENS[name]) throw new Error(
    `Missing token ${name}; available: ${Object.keys(TOKENS).join(' ')}`
  )
  return TOKENS[name].asCustomToken
}

async function tokensFromName ({
  run,
  TOKENS, name = 'UNKNOWN-UNKNOWN'
}): Promise<{token0: TokenType, token1: TokenType}> {
  const [name0, name1] = name.split('-')
  return {
    token0: await run(tokenFromName, { TOKENS, name: name0 }),
    token1: await run(tokenFromName, { TOKENS, name: name1 })
  }
}

export type SupportedTokens = Record<string, AMMSNIP20Client>

async function getSupportedTokens ({
  agent, deployment, run,
  settings: { swapTokens } = getSettings(agent.chain.id),
  TOKENS = { SIENNA: new AMMSNIP20Client({...deployment.get('SIENNA'), agent}) }
}): Promise<SupportedTokens> {
  // On localnet, support placeholder tokens
  if (agent.chain.isLocalnet) {
    // On localnet, deploy some placeholder tokens corresponding to the config.
    await run(getPlaceholderTokens, { TOKENS })
  }
  // On testnet and mainnet, use preexisting token contracts specified in the config.
  if (!agent.chain.isLocalnet) {
    await run(getConfiguredTokens, { TOKENS })
  }
  return TOKENS
}

async function getConfiguredTokens ({
  agent,
  settings = getSettings(agent.chain.id),
  TOKENS
}): Promise<SupportedTokens> {
  console.info(`Not running on localnet, using tokens from config:`)
  const swapTokens: Record<string, { address: string, codeHash: string }> = settings.swapTokens
  await Promise.all(
    Object.entries(swapTokens).map(
      async ([name, {address, codeHash}])=>{
        codeHash = await agent.checkCodeHash(address, codeHash) 
        TOKENS[name] = new AMMSNIP20Contract({ address, codeHash }).client(agent)
      }
    )
  )
  return TOKENS
}

async function getPlaceholderTokens ({
  agent, deployment,
  settings = getSettings(agent.chain.id),
  TOKENS   = {}
}): Promise<SupportedTokens> {

  const toDeploy = []

  const placeholders: Record<string, { label: string, initMsg: any }> = settings.placeholderTokens

  // collect list of missing placeholder tokens to deploy
  for (const [_, {label, initMsg}] of Object.entries(placeholders)) {
    const { symbol } = initMsg
    if (TOKENS[symbol]) {
      console.info(bold(symbol), 'exists in working memory')
      continue
    }
    const name = `Placeholder.${symbol}`
    if (deployment.receipts[name]) {
      console.info(bold(name), 'exists in current deployment')
      const receipt = deployment.get(name)
      TOKENS[symbol] = new AMMSNIP20Client({...receipt, agent})
      continue
    }
    console.info(bold(name), 'deploying...')
    const TOKEN = new AMMSNIP20Contract({ prefix: deployment.prefix, name })
    TOKEN.name = name
    await agent.chain.buildAndUpload(agent, [TOKEN])
    toDeploy.push([symbol, TOKEN, { ...initMsg, name: initMsg.symbol }])
  }

  if (toDeploy.length > 0) {

    // deploy all placeholders in 1 tx
    let bundle = agent.bundle()
    for (let [symbol, TOKEN, initMsg] of toDeploy) {
      initMsg = {...TOKEN.initMsg, ...initMsg, admin: agent.address}
      bundle = bundle.init(TOKEN.template, TOKEN.label, initMsg)
    }
    const { logs, transactionHash } = await bundle.run()

    // bundling api is virtually nonexistent
    // gotta fish out the addresses manually
    for (const i in logs) {
      const [symbol, TOKEN] = toDeploy[i]
      deployment.receipts[TOKEN.name] = TOKEN.instance = {
        chainId:  TOKEN.template.chainId,
        codeId:   TOKEN.template.codeId,
        codeHash: TOKEN.template.codeHash,
        address:  logs[i].events[0].attributes[4].value,
        transactionHash
      }
      deployment.save()
      TOKENS[symbol] = TOKEN.client(agent)
    }

    // mint test balances for all placeholders in 1 tx
    await agent.bundle().wrap(async bundle=>{
      for (const [symbol, _, __] of toDeploy) {
        const amount = "100000000000000000000000"
        console.warn("Minting", bold(amount), bold(symbol), 'to', bold(agent.address))
        const {address, codeHash} = TOKENS[symbol]
        const token = new AMMSNIP20Client({address, codeHash, agent: bundle})
        await token.setMinters([agent.address])
        await token.mint(amount, agent.address)
      }
    })
  }

  return TOKENS

}

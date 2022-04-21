# Sienna: Helper Functions Concerning Tokens

```typescript
import { Console, bold } from '@hackbg/fadroma'
const console = new Console('@sienna/scripts/Tokens')
```

## Get token descriptors (addr+hash) from token names

```typescript
/** Convert pair in format TOKEN0-TOKEN1
  * to token pair descriptor. */
export function fromPairName (
  knownTokens, name = 'UNKNOWN-UNKNOWN'
): Promise<{token0: TokenType, token1: TokenType}> {
  const [name0, name1] = name.split('-')
  return {
    token0: fromName(knownTokens, name0),
    token1: fromName(knownTokens, name1)
  }
}

/** Convert token name to token descriptor. */
export function fromName (knownTokens, name = 'UNKNOWN'): TokenType {
  if (!knownTokens[name]) throw new Error(
    `Missing token ${name}; available: ${Object.keys(knownTokens).join(' ')}`
  )
  return knownTokens[name].asCustomToken
}
```

## Get all tokens supported by the DEX

```typescript
import * as API from '@sienna/api'
import getSettings from '@sienna/settings'

export type SupportedTokens = Record<string, API.AMMSnip20Client>

/** Get a collection of all supported tokens, including placeholders if devnet. */
export async function getSupported ({
  agent, deployment, run,
  swapTokens  = getSettings(agent.chain.mode).swapTokens,
  knownTokens = {
    SIENNA: new API.AMMSnip20Client({...deployment.get('SIENNA'), agent })
  }
}): Promise<SupportedTokens> {
  // On devnet, support placeholder tokens
  if (agent.chain.isDevnet) {
    // On devnet, deploy some placeholder tokens corresponding to the config.
    await run(getOrCreatePlaceholderTokens, { knownTokens })
  }
  // On testnet and mainnet, use preexisting token contracts specified in the config.
  if (!agent.chain.isDevnet) {
    await run(getConfiguredTokens, { knownTokens })
  }
  return knownTokens
}

/** Get handles to tokens from swapTokens. */
export async function getConfiguredTokens ({
  agent,
  settings = getSettings(agent.chain.mode),
  knownTokens
}): Promise<SupportedTokens> {
  console.info(`Not running on devnet, using tokens from config:`)
  const swapTokens: Record<string, { address: string, codeHash: string }> = settings.swapTokens
  await Promise.all(
    Object.entries(swapTokens).map(
      async ([name, {address, codeHash}])=>{
        codeHash = await agent.checkCodeHash(address, codeHash)
        knownTokens[name] = new API.AMMSnip20Client({ agent, address, codeHash })
      }
    )
  )
  return knownTokens
}
```

## Deploy placeholder tokens

Used by the DEX on devnet because ephemeral devnets don't contain
the "official" testnet tokens.

```typescript
import { randomHex } from '@hackbg/fadroma'
import { source, sources, versions, contracts } from './Build'

/** Get a collection of the placeholder tokens to use on devnet. */
export async function getOrCreatePlaceholderTokens (context: MigrationContext & {
  /** The settings should contain a list of placeholder tokens to create. */
  placeholders: Record<string, { label: string, initMsg: any }>
  /** A collection of pre-existing tokens.
    * Will be mutated to contain newly deployed placeholder tokens. */
  knownTokens:  Record<string, any>
  /** Template for placeholder token. Default: AMMSnip20 */
  template:     Template
}): Promise<SupportedTokens> {

  const {
    chain, deployAgent, clientAgent,
    uploader,
    deployment,
    placeholders = getSettings(chain.mode).placeholderTokens,
    knownTokens  = {}
    builder      = new Scrt_1_2.Builder()
    template     = await uploader.upload(await builder.build(source('amm-snip20')))
    admin        = clientAgent.address,
  } = context

  // 1. Iterate over the list of placeholders in the settings,
  //    and collect the ones to be created in the following array:
  const createPlaceholders: [name, InitMsg][] = []
  for (const [_, {label, initMsg}] of Object.entries(placeholders)) {

    // Tokens are keyed by symbol.
    const { symbol } = initMsg

    // If the token is already in the collection, do nothing.
    if (knownTokens[symbol]) {
      console.info(bold(symbol), 'exists in working memory')
      continue
    }

    // If the token is not in the collection,
    // try to find it from the deployment.
    const name = `Placeholder.${symbol}`
    if (deployment.receipts[name]) {
      console.info(bold(name), 'exists in current deployment')
      const receipt = deployment.get(name)
      knownTokens[symbol] = new API.AMMSnip20Client({...receipt, agent: clientAgent})
      continue
    }

    // If there's no such token in the deployment, either,
    // add it to a list of tokens to deploy.
    console.info(bold('Placeholder:'), symbol, '- will deploy...')
    createPlaceholders.push([name, {
      prng_seed: randomHex(36),
      decimals:  18,
      config: { public_total_supply: true, enable_mint: true },
      ...initMsg,
      admin
    }])

  }

  if (createPlaceholders.length > 0) {

    // 2. Deploy missing placeholder tokens in 1 tx
    const createdPlaceholders =
      await deployment.initMany(deployAgent, template, createPlaceholders)

    // 3. Contracts are deployed with admin set to clientAgent,
    //    so we create this next bundle from clientAgent, too:
    await clientAgent.bundle().wrap(async clientBundle=>{
      for (const i in createPlaceholders) {

        // 4. Collect the instantiated placeholder token in knownTokens
        const [_, {name, symbol}] = createPlaceholders[i]
        const instance = createdPlaceholders[i]
        const client = new API.AMMSnip20Client({...instance, agent: clientBundle})
        knownTokens[symbol] = client

        // 5. Mint test balance in placeholder token for the admin
        const amount = "100000000000000000000000"
        console.warn("Minting", bold(amount), bold(name), 'to', bold(admin))
        await client.setMinters([admin])
        await client.mint(amount, admin)

      }
    })

  }

  return knownTokens

}
```

import {
  Console, bold, timestamp, randomHex, SNIP20Contract,
  printContract, printContracts
} from '@hackbg/fadroma'
const console = Console('@sienna/factory/Deploy')

import getSettings from '@sienna/settings'

import { deployPlaceholders } from '@sienna/amm-snip20'

export async function deployAMMExchanges ({
  run, chain, agent,
  SIENNA,
  FACTORY,
  version,
  settings: { swapTokens, swapPairs } = getSettings(chain.id),
}) {
  // Collect referenced tokens, and created exchanges/LPs
  const TOKENS:    Record<string, SNIP20Contract> = { SIENNA }
  const EXCHANGES: AMMExchangeContract[]     = []
  const LP_TOKENS: LPTokenContract[] = []
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
      const realCodeHash = await agent.getCodeHash(address)
      if (codeHash !== realCodeHash) {
        console.warn(bold('Code hash mismatch for'), address, `(${name})`)
        console.warn(bold('  Config:'), codeHash)
        console.warn(bold('  Chain: '), realCodeHash)
      } else {
        console.info(bold(`Code hash of ${address}:`), realCodeHash)
      }
      tokens[name] = new AMMSNIP20Contract({address, codeHash: realCodeHash, agent})
    }
    Object.assign(TOKENS, tokens)
  }
  // If there are any initial swap pairs defined in the config
  for (const name of swapPairs) {
    // Call the factory to deploy an EXCHANGE for each
    const { EXCHANGE, LP_TOKEN } = await run(deployAMMExchange, {
      FACTORY, TOKENS, name, version
    })
    // And collect the results
    EXCHANGES.push(EXCHANGE)
    LP_TOKENS.push(LP_TOKEN)
    //await agent.nextBlock
  }
  return { TOKENS, LP_TOKENS, EXCHANGES }
}

/** Deploy a new exchange. If the exchange already exists, do nothing.
  * Factory doesn't allow 2 identical exchanges to exist (as compared by TOKEN0 and TOKEN1). */
export async function deployAMMExchange ({
  agent, deployment,
  FACTORY, TOKENS, name, version
}) {
  console.info(bold(`Deploying AMM exchange`), name)
  const [tokenName0, tokenName1] = name.split('-')
  if (!TOKENS[tokenName0]) throw new Error(
    `Missing token ${tokenName0}; available: ${Object.keys(TOKENS).join(' ')}`
  )
  const token0 = TOKENS[tokenName0].asCustomToken
  //console.info(`- Token 0: ${bold(JSON.stringify(token0))}...`)
  if (!TOKENS[tokenName1]) throw new Error(
    `Missing token ${tokenName1}; available: ${Object.keys(TOKENS).join(' ')}`
  )
  const token1 = TOKENS[tokenName1].asCustomToken
  //console.info(`- Token 1: ${bold(JSON.stringify(token1))}...`)
  try {
    const { EXCHANGE, LP_TOKEN } = await FACTORY.getExchange(token0, token1)
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

/** Since exchange and LP token are deployed through the factory
  * and not though Fadroma Deploy, we need to manually save their
  * addresses in the deployment. */ 
function saveExchange (
  { deployment, version },
  { pair_contract: { id: ammId, code_hash: ammHash }, lp_token_contract: { id: lpId } },
  { name, raw, EXCHANGE, LP_TOKEN, TOKEN_0, TOKEN_1 }
) {
  console.info(bold(`Deployed AMM exchange`), EXCHANGE.address)
  const ammReceipt = {
    ...raw, codeId: ammId, codeHash: ammHash,
    initTx: { contractAddress: raw.exchange.address }
  } 
  deployment.save(ammReceipt, `SiennaSwap_${version}_${name}`)
  console.info(bold(`Deployed LP token`), LP_TOKEN.address)
  const lpReceipt = {
    ...raw, codeId: lpId, codeHash: raw.lp_token.code_hash,
    initTx:   { contractAddress: raw.lp_token.address }
  }
  deployment.save(lpReceipt, `SiennaSwap_${version}_LP-${name}`)
  return { name, raw, EXCHANGE, LP_TOKEN, TOKEN_0, TOKEN_1 }
}

export async function printExchanges (EXCHANGES?: any[]) {
  if (!EXCHANGES) {
    console.info('No exchanges found.')
    return
  }
  for (const { name, EXCHANGE, TOKEN_0, TOKEN_1, LP_TOKEN } of EXCHANGES) {
    const { codeId, codeHash, address } = EXCHANGE
    console.info(
      ' ', bold(colors.inverse(name)).padEnd(30), // wat
      `(code id ${bold(String(codeId))})`.padEnd(34), bold(address)
    )
    await printToken(TOKEN_0)
    await printToken(TOKEN_1)
    await printToken(LP_TOKEN)
  }
}

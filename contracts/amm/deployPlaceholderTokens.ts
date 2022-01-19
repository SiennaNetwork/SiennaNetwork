import { IChain, IAgent, timestamp, randomHex, buildAndUpload } from '@fadroma/scrt'
import type { SNIP20Contract } from "@fadroma/snip20"

import { AMMSNIP20Contract } from '@sienna/api'
import settings, { workspace } from '@sienna/settings'

export type PlaceholderTokenConfig = Record<string, {
  label:   string,
  initMsg: any
}>

export async function deployPlaceholderTokens ({
  chain,
  admin,
  deployment: { name: prefix, contracts }
}: {
  chain:      IChain,
  admin:      IAgent,
  deployment: { name: string, contracts: Record<string, any> }
}): Promise<Record<string, SNIP20Contract>> {

  const AMMTOKEN = new AMMSNIP20Contract({ workspace, prefix, chain, admin })
  await buildAndUpload([AMMTOKEN])
  const placeholders: PlaceholderTokenConfig = settings(chain.chainId).placeholderTokens
  const tokens = {}

  for (const [symbol, {label, initMsg}] of Object.entries(placeholders)) {

    const token = tokens[symbol] = new AMMSNIP20Contract({
      chain,
      admin,
      instantiator: admin,
      codeId:   AMMTOKEN.codeId,
      codeHash: AMMTOKEN.codeHash,
      prefix,
      suffix: `_${label}+${timestamp()}`,
      initMsg: { ...initMsg, prng_seed: randomHex(36) }
    })

    const existing = contracts[label]
    await tokens[symbol].instantiateOrExisting(existing)
    await tokens[symbol].tx(admin).setMinters([admin.address])
    await tokens[symbol].tx(admin).mint("100000000000000000000000", admin.address)

  }

  return tokens

}

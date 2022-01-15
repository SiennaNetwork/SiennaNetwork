import { randomHex } from '@hackbg/tools'
import type { IChain, IAgent } from '@fadroma/scrt'
import type { SNIP20Contract } from "@fadroma/snip20";

import { AMMSNIP20Contract } from '@sienna/api'
import settings from '@sienna/settings'

export type PlaceholderTokenOptions = {
  chain:    IChain,
  admin:    IAgent,
  prefix:   string,
  instance: { contracts: Record<string, any> }
}

export type PlaceholderTokenConfig = Record<string, {
  label:   string,
  initMsg: any
}>

export default async function deployPlaceholderTokens ({
  chain, admin, prefix, instance
}: PlaceholderTokenOptions): Promise<Record<string, SNIP20Contract>> {
  const AMMTOKEN = new AMMSNIP20Contract({ prefix, admin })
  const placeholders: PlaceholderTokenConfig = settings(chain.chainId).placeholderTokens
  const tokens = {}
  for (const [symbol, {label, initMsg}] of Object.entries(placeholders)) {
    const token = tokens[symbol] = new AMMSNIP20Contract({ admin })
    Object.assign(token.blob, { codeId: AMMTOKEN.codeId, codeHash: AMMTOKEN.codeHash })
    Object.assign(token.init, { prefix, label, msg: initMsg })
    Object.assign(token.init.msg, { prng_seed: randomHex(36) })
    const existing = instance.contracts[label]
    await tokens[symbol].instantiateOrExisting(existing)
    await tokens[symbol].setMinters([admin.address], admin)
    await tokens[symbol].mint("100000000000000000000000", admin)
  }
  return tokens
}

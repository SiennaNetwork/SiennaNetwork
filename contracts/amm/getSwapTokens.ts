import { IAgent } from '@hackbg/fadroma'
import { SNIP20Contract } from '@fadroma/snip20'
import { AMMSNIP20Contract } from '@sienna/api'

export function getSwapTokens (
  links: Record<string, { address: string, codeHash: string }>,
  admin?: IAgent
): Record<string, SNIP20Contract> {
  const tokens = {}
  for (const [name, {address, codeHash}] of Object.entries(links)) {
    tokens[name] = new AMMSNIP20Contract({address, codeHash, admin})
    console.log('getSwapToken', name, address, codeHash)
  }
  return tokens
}

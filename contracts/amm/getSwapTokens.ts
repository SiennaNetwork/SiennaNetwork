import { AMMSNIP20Contract } from '@sienna/api'

export function getSwapTokens (links: Record<string, { address: string, codeHash: string }>) {
  const tokens = {}
  for (const [name, {address, codeHash}] of Object.entries(links)) {
    tokens[name] = new AMMSNIP20Contract({
      address,
      codeHash,
      admin
    })
    console.log('getSwapToken', name, address, codeHash)
  }
  return tokens
}

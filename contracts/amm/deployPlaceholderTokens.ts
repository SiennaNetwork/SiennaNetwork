import { Migration, randomHex } from '@hackbg/fadroma'
import type { SNIP20Contract } from "@fadroma/snip20"

import { AMMSNIP20Contract } from '@sienna/api'
import settings, { workspace } from '@sienna/settings'

export type TokenConfig = { label: string, initMsg: any }

export async function deployPlaceholderTokens (options: Migration): Promise<
  Record<string, SNIP20Contract>
> {

  const {

    timestamp,

    chain,
    admin,

    prefix,
    contracts

  } = options

  const AMMTOKEN = new AMMSNIP20Contract({ workspace, prefix, chain, admin })
  await chain.buildAndUpload([AMMTOKEN])

  const placeholders: Record<string, TokenConfig> =
    settings(chain.chainId).placeholderTokens

  const tokens = {}

  for (const [symbol, {label, initMsg}] of Object.entries(placeholders)) {

    const token = tokens[symbol] = new AMMSNIP20Contract({
      chain,
      admin,
      instantiator: admin,
      codeId:       AMMTOKEN.codeId,
      codeHash:     AMMTOKEN.codeHash,
      prefix,
      suffix:       `_${label}+${timestamp}`,
      initMsg: { ...initMsg, prng_seed: randomHex(36) }
    })

    const existing = contracts[label]
    await tokens[symbol].instantiateOrExisting(existing)
    await tokens[symbol].tx(admin).setMinters([admin.address])
    await tokens[symbol].tx(admin).mint("100000000000000000000000", admin.address)

  }

  return tokens

}

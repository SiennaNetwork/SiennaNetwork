import { Migration, randomHex } from '@hackbg/fadroma'
import type { SNIP20Contract } from "@fadroma/snip20"

import { AMMSNIP20Contract } from '@sienna/api'
import settings, { workspace } from '@sienna/settings'

export type TokenConfig = { label: string, initMsg: any }

export async function deployPlaceholderTokens (options: Migration)
  : Promise<Record<string, SNIP20Contract>>
{

  const {
    deployment,
    // TODO: fold these three into the deployment
    chain,
    admin,
    prefix,
    // TODO: and this one makes a fine foundation for a multi-stage migration system
    timestamp,
  } = options

  const AMMTOKEN = new AMMSNIP20Contract({ workspace, prefix, chain, admin })

  // this can later be used to check if the deployed contracts have
  // gone out of date (by codehash) and offer to redeploy them
  await chain.buildAndUpload([AMMTOKEN])

  const { placeholderTokens } = settings(chain.chainId)
  const placeholders: Record<string, TokenConfig> = placeholderTokens

  const tokens = {}

  for (const [symbol, {label: suffix, initMsg}] of Object.entries(placeholders)) {

    const TOKEN = tokens[symbol] = new AMMSNIP20Contract({
      chain,
      admin,
      instantiator: admin,
      codeId:       AMMTOKEN.codeId,
      codeHash:     AMMTOKEN.codeHash,
      prefix,
      name:         'AMMSNIP20',
      suffix:       `_${suffix}+${timestamp}`,
      initMsg: { ...initMsg, prng_seed: randomHex(36) }
    })

    // the instantiateOrExisting mechanic needs work -
    // chiefly, to decide in which subsystem it lives.
    // probably move that into `deployment` as well.
    // or, Deployment's child - Migration proper,
    // represented by a single JSON file containing
    // all the inputs and outputs of one of these.
    const existing = deployment.contracts[`AMMSNIP20_${suffix}`]
    await TOKEN.instantiateOrExisting(existing)

    // newly deployed placeholder tokens give the admin a large balance.
    // these are only intended for localnet so when you run out of it
    // it's a good time to redeploy the localnet to see if all is in order anyway.
    if (!existing) {
      await TOKEN.tx(admin).setMinters([admin.address])
      await TOKEN.tx(admin).mint("100000000000000000000000", admin.address)
    }

  }
  return tokens
}

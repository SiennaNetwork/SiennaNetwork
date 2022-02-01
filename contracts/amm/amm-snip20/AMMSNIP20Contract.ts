import { Agent, MigrationContext, randomHex, bold, timestamp, Console } from '@hackbg/fadroma'
import { SNIP20Contract, SNIP20Contract_1_2 } from '@fadroma/snip20'
import getSettings, { workspace } from '@sienna/settings'
import { InitMsg } from './schema/init_msg.d'

const console = Console('@sienna/amm-snip20')

export class AMMSNIP20Contract extends SNIP20Contract_1_2 {
  crate = 'amm-snip20'
  name  = 'AMMSNIP20'
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
  constructor (options) {
    super(options)
    const { name, agent } = options||{}
    if (name) this.name = name // why
    if (agent) this.agent = agent // why
  }
}

export async function deployPlaceholders ({ chain, admin, deployment, prefix }) {
  // this can later be used to check if the deployed contracts have
  // gone out of date (by codehash) and offer to redeploy them
  const PLACEHOLDERS = {}
  const { placeholderTokens } = getSettings(chain.chainId)
  console.info(
    bold(`Deploying placeholder tokens`), Object.keys(placeholderTokens).join(' ')
  )
  type TokenConfig = { label: string, initMsg: any }
  const placeholders: Record<string, TokenConfig> = placeholderTokens
  for (const [_, {label, initMsg}] of Object.entries(placeholders)) {
    const name = initMsg.symbol
    try {
      PLACEHOLDERS[name] = deployment.getThe(`Placeholder_${label}`, new AMMSNIP20Contract({}))
      console.info(bold('Found, not redeploying:'), name)
    } catch (e) {
      if (e.message.startsWith('@fadroma/ops: no contract')) {
        console.info(bold('Deploying placeholder:'), name)
        const TOKEN = PLACEHOLDERS[name] = new AMMSNIP20Contract({
          workspace, prefix, name: `Placeholder_${label}`, suffix: `+${timestamp()}`,
        })
        await chain.buildAndUpload(admin, [TOKEN])
        await deployment.createContract(admin, TOKEN, { ...initMsg, name })
        await TOKEN.tx().setMinters([admin.address])
        await TOKEN.tx().mint("100000000000000000000000", admin.address)
      } else {
        console.error(e)
        throw new Error(
          `@sienna/amm/deploy: error when deploying placeholder tokens: ${e.message}`
        )
      }
    }
  }
  return { PLACEHOLDERS }
}

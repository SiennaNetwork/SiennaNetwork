import { Agent, MigrationContext, randomHex, bold, timestamp, Console } from '@hackbg/fadroma'
import { SNIP20Contract, SNIP20Contract_1_2 } from '@hackbg/fadroma'
import getSettings, { workspace } from '@sienna/settings'
import { InitMsg } from './schema/init_msg.d'

const console = Console('@sienna/amm-snip20')

export class AMMSNIP20Contract extends SNIP20Contract_1_2 {
  workspace = workspace
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

/** On localnet, the AMM exchanges use these tokens. */
export async function deployPlaceholders ({ chain, agent, deployment, prefix }) {

  const { placeholderTokens } = getSettings(chain.id)
  type TokenConfig = { label: string, initMsg: any }
  const placeholders: Record<string, TokenConfig> = placeholderTokens

  const PLACEHOLDERS = {}

  for (const [_, {label, initMsg}] of Object.entries(placeholders)) {
    const name = `Placeholder.${label}`
    try {
      PLACEHOLDERS[initMsg.symbol] = deployment.getThe(name, new AMMSNIP20Contract({}))
    } catch (e) {
      if (e.message.startsWith('@fadroma/ops: no contract')) {
        const TOKEN = PLACEHOLDERS[initMsg.symbol] = new AMMSNIP20Contract({ prefix, name })
        await chain.buildAndUpload(agent, [TOKEN])
        await deployment.init(agent, TOKEN, { ...initMsg, name: initMsg.symbol })
      } else {
        console.error(e)
        throw new Error(
          `@sienna/amm/deploy: error when deploying placeholder tokens: ${e.message}`
        )
      }
    }
  }

  await agent.bundle(async agent=>{
    for (const [_, {label, initMsg}] of Object.entries(placeholders)) {
      const name = `Placeholder.${label}`
      const amount = "100000000000000000000000"
      console.warn("Minting", bold(amount), name, 'to', bold(agent.address))
      const {address} = agent
      const token = PLACEHOLDERS[initMsg.symbol].tx(agent)
      await token.setMinters([address])
      await token.mint(amount, address)
    }
  })

  return { PLACEHOLDERS }
}

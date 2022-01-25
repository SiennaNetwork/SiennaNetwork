import { Migration, bold, writeFileSync, Console } from '@hackbg/fadroma'
import { SNIP20Contract } from '@fadroma/snip20'
import { FactoryContract } from '@sienna/api'

const console = Console('@sienna/amm/deployLiquidityPool')

export async function deployLiquidityPool (options: Migration & {
  name:    string
  tokens:  Record<string, SNIP20Contract>
  FACTORY: FactoryContract
}) {

  const {
    deployment,
    admin,

    name,
    tokens,
    FACTORY,
  } = options

  console.info(`Deploying liquidity pool ${bold(name)}...`)

  const [tokenName0, tokenName1] = name.split('-')
  const token0 = tokens[tokenName0].asCustomToken
  const token1 = tokens[tokenName1].asCustomToken

  console.info(`- Token 0: ${bold(JSON.stringify(token0))}...`)
  console.info(`- Token 1: ${bold(JSON.stringify(token1))}...`)

  try {

    const exchange = await FACTORY.getExchange(token0, token1, admin)
    console.info(`${bold(name)}: Already exists.`)
    return exchange

  } catch (e) {

    if (e.message.includes("Address doesn't exist in storage")) {
      const deployed = await FACTORY.createExchange(token0, token1)
      deployment.save(deployed, `SiennaSwap_${name}`)
      console.info(
        `Deployed liquidity pool ${deployed.exchange.address} `+
        ` and LP token ${deployed.lp_token.address}`
      )
      return deployed
    } else {
      throw new Error(`${bold(`Factory::GetExchange(${name})`)}: not found (${e.message})`)
    }

  }

}

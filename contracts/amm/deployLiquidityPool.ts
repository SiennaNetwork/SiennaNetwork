import { Migration, bold, writeFileSync } from '@hackbg/fadroma'
import { SNIP20Contract } from '@fadroma/snip20'
import { FactoryContract } from '@sienna/api'

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

  const [tokenName0, tokenName1] = name.split('-')
  const token0 = tokens[tokenName0].asCustomToken
  const token1 = tokens[tokenName1].asCustomToken
  console.log(`\nLiquidity pool ${bold(name)}...`)

  try {

    const exchange = await FACTORY.getExchange(token0, token1, admin)
    console.info(`${bold(name)}: Already exists.`)
    return exchange

  } catch (e) {

    if (e.message.includes("Address doesn't exist in storage")) {
      console.info(`${bold(`FACTORY.getExchange(${name})`)}: not found (${e.message}), deploying...`)
      const deployed = await FACTORY.createExchange(token0, token1)
      deployment.save(deployed, `SiennaSwap_${name}.json`)
      console.info(bold('Deployed.'), deployed)
      return deployed
    } else {
      throw new Error(`${bold(`FACTORY.getExchange(${name})`)}: not found (${e.message})`)
    }

  }

}

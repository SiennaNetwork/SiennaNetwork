import { Migration, bold, writeFileSync } from '@hackbg/fadroma'
import { SNIP20Contract } from '@fadroma/snip20'
import { FactoryContract } from '@sienna/api'

export async function deployLiquidityPool (options: Migration & {
  name:    string
  tokens:  Record<string, SNIP20Contract>
  FACTORY: FactoryContract
}) {

  const {
    saveReceipt,

    name,
    tokens,
    FACTORY,
    admin,
  } = options

  const [tokenName0, tokenName1] = name.split('-')
  const token0 = tokens[tokenName0]
  const token1 = tokens[tokenName1]
  console.log(`\nLiquidity pool ${bold(name)}...`)

  try {

    const exchange = await FACTORY.getExchange(
      token0.asCustomToken,
      token1.asCustomToken,
      admin
    )
    console.info(`${bold(name)}: Already exists.`)
    return exchange

  } catch (e) {

    if (e.message.includes("Address doesn't exist in storage")) {
      console.info(`${bold(`FACTORY.getExchange(${name})`)}: not found (${e.message}), deploying...`)
      const deployed = await FACTORY.createExchange(
        token0.asCustomToken,
        token1.asCustomToken
      )
      saveReceipt(`SiennaSwap_${name}.json`, deployed)
      console.info(bold('Deployed.'), deployed)
      return deployed
    } else {
      throw new Error(`${bold(`FACTORY.getExchange(${name})`)}: not found (${e.message})`)
    }

  }

}

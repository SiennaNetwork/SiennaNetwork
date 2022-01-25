import { Migration } from '@hackbg/fadroma'
import { FactoryContract } from '@sienna/api'
import settings from '@sienna/settings'

export async function deployLegacyFactory (options: Migration) {

  const {
    timestamp,
    chain,
    prefix,
    getContract,
    admin
  } = options

  const FACTORY = getContract(FactoryContract, 'SiennaAMMFactory', admin)

  const V1_FACTORY = new FactoryContract({
    ref:    `main`,
    prefix,
    suffix: `@v1+${timestamp}`,
    admin,
    exchange_settings: settings(chain.id).amm.exchange_settings,
    contracts:         await FACTORY.contracts,
  })

  await chain.buildAndUpload([V1_FACTORY])
  await V1_FACTORY.instantiate()

}

import { timestamp } from '@hackbg/tools'
import type { IChain, IAgent, Deployment } from '@fadroma/scrt'
import { buildAndUpload, Scrt } from '@fadroma/scrt'

import { FactoryContract } from '@sienna/api'
import settings from '@sienna/settings'

export async function deployLegacyFactory (chain: IChain, admin: IAgent) {
  const deployment = chain.deployments.active
  const prefix     = deployment.name
  const V2_FACTORY = deployment.getContract(FactoryContract, 'SiennaAMMFactory', admin)
  const contracts  = await V2_FACTORY.contracts
  const V1_FACTORY = new FactoryContract({
    prefix,
    admin,
    exchange_settings: settings(chain.chainId).amm.exchange_settings,
    contracts:         await V2_FACTORY.contracts,
    ref:               `main`,
    suffix:            `@v1+${timestamp()}`
  })
  await buildAndUpload([V1_FACTORY])
  await V1_FACTORY.instantiate()
}

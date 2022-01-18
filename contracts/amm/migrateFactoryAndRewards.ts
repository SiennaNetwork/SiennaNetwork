import { timestamp } from '@hackbg/tools'
import type { IChain, IAgent, Deployment } from '@fadroma/scrt'
import { buildAndUpload, Scrt } from '@fadroma/scrt'

import { FactoryContract } from '@sienna/api'
import settings from '@sienna/settings'

type MultisigTX = any

export async function migrateFactoryAndRewards (chain: IChain, admin: IAgent): Promise<MultisigTX[]> {
  const deployment = chain.deployments.active
  const V2_FACTORY = deployment.getContract(FactoryContract, 'SiennaAMMFactory', admin)
  const V1_FACTORY = new FactoryContract({
    ref:    `main`,
    prefix: deployment.name,
    suffix: `@v1+${timestamp()}`,
    admin,
    exchange_settings: settings(chain.chainId).amm.exchange_settings,
    contracts:         await V2_FACTORY.contracts,
  })
  await buildAndUpload([V1_FACTORY])
  await V1_FACTORY.instantiate()
  return []
}

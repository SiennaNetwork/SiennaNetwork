import { timestamp } from '@hackbg/tools'
import type { IChain, IAgent, Deployment } from '@fadroma/scrt'
import { buildAndUpload, Scrt } from '@fadroma/scrt'

import {
  FactoryContract,
  AMMContract,
  LPTokenContract
} from '@sienna/api'
import settings from '@sienna/settings'

type MultisigTX = any

export async function migrateFactoryAndRewards (chain: IChain, admin: IAgent): Promise<MultisigTX[]> {

  const deployment = chain.deployments.active

  const OLD_FACTORY:   FactoryContract   =
    deployment.getContract(FactoryContract, 'SiennaAMMFactory', admin)
  const OLD_EXCHANGES: AMMContract[]     =
    []
  const OLD_LP_TOKENS: LPTokenContract[] =
    []
  const OLD_REWARDS:   RewardsContract[] =
    []

  const NEW_FACTORY = new FactoryContract({
    ref:    `main`,
    prefix: deployment.name,
    suffix: `@v2.0.0+${timestamp()}`,
    admin,
    exchange_settings: settings(chain.chainId).amm.exchange_settings,
    contracts:         await V2_FACTORY.contracts,
  })
  const NEW_EXCHANGES: AMMContract[]     =
    []
  const NEW_LP_TOKENS: LPTokenContract[] =
    []
  const NEW_REWARDS:   RewardsContract[] =
    []

  return []
}

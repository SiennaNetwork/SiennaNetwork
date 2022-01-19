import { timestamp } from '@hackbg/tools'
import type { IChain, IAgent, Deployment } from '@fadroma/scrt'
import { buildAndUpload, Scrt } from '@fadroma/scrt'
import type { SNIP20Contract } from '@fadroma/snip20'
import { FactoryContract, AMMContract, LPTokenContract, RewardsContract } from '@sienna/api'
import settings from '@sienna/settings'

type MultisigTX = any

export async function migrateFactoryAndRewards (chain: IChain, admin: IAgent): Promise<MultisigTX[]> {

  const deployment = chain.deployments.active

  const OLD_FACTORY:   FactoryContract   =
    deployment.getContract(FactoryContract, 'SiennaAMMFactory', admin)
  const OLD_EXCHANGES: AMMContract[]     =
    await OLD_FACTORY.exchanges
  const OLD_LP_TOKENS: SNIP20Contract[] =
    OLD_EXCHANGES.map(exchange=>exchange.lpToken)
  const OLD_REWARDS:   RewardsContract[] =
    deployment.getContracts(RewardsContract, 'SiennaRewards', admin)

  console.log({
    OLD_FACTORY,
    OLD_EXCHANGES,
    OLD_LP_TOKENS,
    OLD_REWARDS
  })

  const NEW_FACTORY = new FactoryContract({
    ref:    `main`,
    prefix: deployment.name,
    suffix: `@v2.0.0+${timestamp()}`,
    admin,
    exchange_settings: settings(chain.chainId).amm.exchange_settings,
    contracts:         await OLD_FACTORY.contracts,
  })
  const NEW_EXCHANGES: AMMContract[]     =
    []
  const NEW_LP_TOKENS: LPTokenContract[] =
    []
  const NEW_REWARDS:   RewardsContract[] =
    []

  return []

}

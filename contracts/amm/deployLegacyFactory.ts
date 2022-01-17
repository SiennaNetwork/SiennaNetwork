import type { IChain, IAdmin, Deployment } from '@fadroma/scrt'
import { buildAndUpload, Scrt } from '@fadroma/scrt'

import {
  AMMContract,
  AMMSNIP20Contract,
  LPTokenContract,
  IDOContract,
  LaunchpadContract,
  FactoryContract
} from '@sienna/api'
import settings from '@sienna/settings'

export async function deployV1FactoryFromV2Factory ({
  chain, admin, deployment
}: {
  chain:      IChain,
  admin:      IAdmin,
  deployment: Deployment
}) {
  const prefix = deployment.name
  const V2_FACTORY = deployment.getContract(FactoryContract, 'SiennaAMMFactory', admin)
  const contracts  = await V2_FACTORY.getContracts()
  const V1_FACTORY = new FactoryContract({
    prefix,
    admin,

    exchange_settings: settings(chain.chainId).amm,

    // okay so here we don't need actual contract instances
    // we only need references to contract contracts
    contracts: await V2_FACTORY.getTemplates()
  })
}

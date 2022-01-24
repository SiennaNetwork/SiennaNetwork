import { MigrationContext } from '@hackbg/fadroma'
import type { SNIP20Contract } from '@fadroma/snip20'

import {
  RPTContract,
  FactoryContract,
  AMMContract,
  LPTokenContract,
  RewardsContract
} from '@sienna/api'

import settings, { workspace } from '@sienna/settings'

type MultisigTX = any

export async function migrateFactoryAndRewards (options: MigrationContext): Promise<MultisigTX[]> {

  const {
    timestamp,

    chain,
    admin,
    deployment,
    prefix,
  } = options

  // the arbiter of it all.
  // will redirect fundsin a 30:70 proportion
  // from old version to new version
  const RPT: RPTContract =
    deployment.getContract(RPTContract, 'SiennaAMMFactory')
  // the v1 factory. we'll terminate this now,
  // so that new pairs cannot be created from the v1 factory.
  const V1_FACTORY: FactoryContract =
    deployment.getContract(FactoryContract, 'SiennaAMMFactory', admin)
  // the liquidity pools of the v1 factory.
  // we'll disincentivise those in RPT now,
  // and eventually terminate them in the next migration.
  const OLD_LIQUIDITY_POOLS: AMMContract[] =
    await V1_FACTORY.exchanges
  // the LP tokens of the liquidity pools of the v1 factory.
  // we'll disincentivise those in RPT now,
  // and eventually terminate them in the next migration.
  const OLD_LP_TOKENS: SNIP20Contract[] =
    OLD_LIQUIDITY_POOLS.map(exchange=>exchange.lpToken)
  // the reward pools attached to some of the LP tokens
  // of the liquidity pools of the v1 factory.
  // we'll disincentivise those in RPT now,
  // and eventually terminate them in the next migration.
  const V2_REWARD_POOLS: RewardsContract[] =
    deployment.getContracts(RewardsContract, 'SiennaRewards', admin)

  const essentials = ({codeId, codeHash, address, label})=>
    ({codeId, codeHash, address, label})

  console.log('V1 factory:')
  console.table(essentials(V1_FACTORY))
  console.log("V1 factory's exchanges (to be disincentivised):")
  console.table(OLD_LIQUIDITY_POOLS.map(essentials))
  console.log("V1 factory's exchanges' LP tokens (to be disincentivised):")
  console.table(OLD_LP_TOKENS.map(essentials))
  console.log("V2 rewards attached to V1 factory's LP tokens (to be disincentivised)")
  console.table(V2_REWARD_POOLS.map(essentials))

  // The new contracts.
  // Their addresses should be added to the frontend.
  const V2_FACTORY: FactoryContract = new FactoryContract({
    workspace,
    prefix,
    suffix: `@v2.0.0+${timestamp}`,
    admin,
    exchange_settings: settings(chain.chainId).amm.exchange_settings,
  })
  const contracts = await V1_FACTORY.getContracts()
  V2_FACTORY.setContracts(contracts)
  await chain.buildAndUpload([V2_FACTORY])
  await V2_FACTORY.instantiate()

  // The new liquidity pools.
  // Their addresses should be added to the frontend.
  const NEW_LIQUIDITY_POOLS: AMMContract[] = []
  for (const { address, token_0, token_1 } of OLD_LIQUIDITY_POOLS) {
    const NEW_LIQUIDITY_POOL = await V2_FACTORY.createExchange(token_0, token_1)
    console.log(`\nOLD LIQUIDITY POOL ${address}`)
    console.log(`between tokens ${JSON.stringify(token_0)}`)
    console.log(`           and ${JSON.stringify(token_1)}`)
    console.log(`becomes NEW LIQUIDITY POOL ${NEW_LIQUIDITY_POOL.address}`)
    console.log({NEW_LIQUIDITY_POOL})
    NEW_LIQUIDITY_POOL.push(NEW_LIQUIDITY_POOL)
    await admin.nextBlock
  }

  process.exit(123)

  // The new LP tokens.
  // Their addresses should be added to the frontend.
  const NEW_LP_TOKENS: LPTokenContract[] =
    OLD_LP_TOKENS.forEach(exchange=>{
      console.log(`\nOld LP token ${exchange.address}`)
      console.log(`of old liquidity pool TODO`)
      console.log(`has become new liquidity pool TODO`)
    })

  // The v3 reward pools.
  // Their addresses should be added to the frontend.
  const V3_REWARD_POOLS: RewardsContract[] =
    V2_REWARD_POOLS.forEach(rewards=>{
      console.log(`\nOld (v2) reward pool ${rewards.address}`)
      console.log(`for old LP token TODO`)
      console.log(`corresponds to new (v3) reward pool TODO`)
      console.log(`for new LP token TODO`)
    })

  return []

}

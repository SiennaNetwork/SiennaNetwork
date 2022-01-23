import {
  Migration,
  IChain, IAgent,
  Scrt, buildAndUpload,
  randomHex, Console
} from '@fadroma/scrt'

import { SNIP20Contract } from "@fadroma/snip20"

import settings from '@sienna/settings'
import {
  SiennaSNIP20Contract,
  FactoryContract,
  RewardsContract,
  AMMSNIP20Contract,
  LPTokenContract
} from '@sienna/api'

import { deployPlaceholderTokens } from './deployPlaceholderTokens'
import { deployRewardPool } from './deployRewardPool'
import { getSwapTokens } from './getSwapTokens'

const console = Console("@sienna/amm/deployRewards")

const SIENNA_DECIMALS = 18
const ONE_SIENNA = BigInt(`1${[...Array(SIENNA_DECIMALS)].map(()=>'0').join('')}`)

export type RPTRecipient = string
export type RPTAmount    = string
export type RPTConfig    = [RPTRecipient, RPTAmount][]
export type RewardsAPIVersion = 'v2'|'v3'

export async function deployRewards (options: Migration & {
  apiVersion?: 'v2'|'v3'
  suffix?:     string
  split?:      number
  ref?:        string
}): Promise<{
  deployedContracts: RewardsContract[]
  rptConfig:         RPTConfig
}> {

  const {
    run,

    chain,
    admin,
    prefix,

    suffix     = '',
    apiVersion = 'v3',
    split      = 1.0,
    ref        = 'HEAD',
    getContract,
  } = options

  const SIENNA  = getContract(SiennaSNIP20Contract, 'SiennaSNIP20',     admin)
  const FACTORY = getContract(FactoryContract,      'SiennaAMMFactory', admin)
  const REWARDS = new RewardsContract({ prefix, admin, ref })
  await buildAndUpload([REWARDS])

  const tokens = {
    SIENNA,
    ...chain.isLocalnet
      ? await run(deployPlaceholderTokens)
      : getSwapTokens(settings(chain.chainId).swapTokens, admin)
  }

  const deployedContracts = []
  const rptConfig = []

  const reward = BigInt(settings(chain.chainId).rewardPairs.SIENNA) / BigInt(1 / split)
  const pool = await run(deployRewardPool, { suffix, lpToken: SIENNA, rewardToken: SIENNA })
  deployedContracts.push(pool)
  rptConfig.push([pool.address, String(reward * ONE_SIENNA)])

  const swapPairs = settings(chain.chainId).swapPairs

  if (swapPairs.length > 0) {
    const existingExchanges = await FACTORY.listExchanges()
    const rewards = settings(chain.chainId).rewardPairs
    for (const name of swapPairs) {
      if (rewards && rewards[name]) {
        const exchangeName = `SiennaSwap_${name}`
        const exchange = deployment.contracts[exchangeName]
        if (!exchange) {
          console.log(`${exchangeName} doesn't exist`)
          process.exit(1)
        }
        const { lp_token } = exchange
        console.debug(`Deploying rewards for ${name}...`, { lp_token })
        const lpToken = new LPTokenContract({
          address:  exchange.lp_token.address,
          codeHash: exchange.lp_token.code_hash,
          admin
        })
        const reward = BigInt(rewards[name]) / BigInt(1 / split)
        const pool   = await run(deployRewardPool, { suffix, lpToken, rewardToken: SIENNA })
        deployedContracts.push(pool)
        rptConfig.push([pool.address, String(reward * ONE_SIENNA)])
      }
    }
  }

  console.debug('Resulting RPT config:', rptConfig)

  return { deployedContracts, rptConfig }

}

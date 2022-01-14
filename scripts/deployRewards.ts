import { randomHex, Console } from '@fadroma/tools'

import type { IChain, IAgent } from '@fadroma/scrt'
import { Scrt } from '@fadroma/scrt'
import { SNIP20Contract } from "@fadroma/snip20";

import {
  SiennaSNIP20Contract,
  FactoryContract,
  RewardsContract,
  AMMSNIP20Contract,
  LPTokenContract
} from '@sienna/api'
import settings from '@sienna/settings'

import buildAndUpload from './buildAndUpload'
import deployPlaceholderTokens from './deployPlaceholderTokens'

const console = Console(import.meta.url)

const SIENNA_DECIMALS = 18
const ONE_SIENNA = BigInt(`1${[...Array(SIENNA_DECIMALS)].map(()=>'0').join('')}`)

export type RewardsOptions = {
  chain?:  IChain
  admin?:  IAgent
  prefix:  string
  suffix?: string
  split?:  number
  ref?:    string
}

export type RPTRecipient = string
export type RPTAmount    = string
export type RPTConfig    = [RPTRecipient, RPTAmount][]
export type RewardsAPIVersion = 'v2'|'v3'

export default async function deployRewards (
  apiVersion: RewardsAPIVersion = 'v3',
  options:    RewardsOptions,
): Promise<{
  deployedContracts: RewardsContract[]
  rptConfig:         RPTConfig
}> {

  const {
    prefix,
    chain  = await new Scrt().ready,
    admin  = await chain.getAgent(),
    suffix = '',
    split  = 1.0,
    ref    = 'HEAD'
  } = options

  const
    instance = chain.instances.active,
    SIENNA   = instance.getContract(SiennaSNIP20Contract, 'SiennaSNIP20', admin),
    FACTORY  = instance.getContract(FactoryContract, 'SiennaAMMFactory', admin),
    REWARDS  = new RewardsContract({ prefix, admin, ref })

  await buildAndUpload([SIENNA, FACTORY, REWARDS])

  const tokens = { SIENNA }
  if (chain.isLocalnet) {
    const placeholderOptions = { chain, admin, prefix, instance }
    Object.assign(tokens, await deployPlaceholderTokens(placeholderOptions))
  } else {
    Object.assign(tokens, getSwapTokens(settings(chain.chainId).swapTokens, admin))
  }

  const deployedContracts = []
  const rptConfig = []

  const reward = BigInt(settings(chain.chainId).rewardPairs.SIENNA) / BigInt(1 / split)
  const pool = await deployRewardPool(`SIENNA${suffix}`, SIENNA, SIENNA)
  deployedContracts.push(pool)
  rptConfig.push([pool.address, String(reward * ONE_SIENNA)])

  const swapPairs = settings(chain.chainId).swapPairs

  if (swapPairs.length > 0) {

    const existingExchanges = await FACTORY.listExchanges()
    const rewards = settings(chain.chainId).rewardPairs

    for (const name of swapPairs) {
      if (rewards && rewards[name]) {
        const exchangeName = `SiennaSwap_${name}`
        const exchange = instance.contracts[exchangeName]
        if (!exchange) {
          console.log(`${exchangeName} doesn't exist`)
          process.exit(1)
        }
        const { lp_token } = exchange
        console.debug(`Deploying rewards for ${name}...`, { lp_token })
        const lpToken = LPTokenContract.attach(
          exchange.lp_token.address,
          exchange.lp_token.code_hash,
          admin
        )
        const reward = BigInt(rewards[name]) / BigInt(1 / split)
        const pool    = await deployRewardPool(`${name}${suffix}`, lpToken, SIENNA)
        deployedContracts.push(pool)
        rptConfig.push([pool.address, String(reward * ONE_SIENNA)])
      }
    }

  }

  console.debug('Resulting RPT config:', rptConfig)

  return { deployedContracts, rptConfig }

  async function deployRewardPool (name: string, lpToken: SNIP20Contract, rewardToken: SNIP20Contract) {

    const {codeId, codeHash} = REWARDS
        , options    = { codeId, codeHash, prefix, name, admin, lpToken, rewardToken, ref }
        , rewardPool = new RewardsContract(options)
        , receipt    = instance.contracts[rewardPool.init.label]

    // override init msg for legacy api
    if (apiVersion === 'v2') {
      rewardPool.init.msg = {
        admin:        admin.address,
        lp_token:     lpToken.link,
        reward_token: rewardToken.link,
        viewing_key:  "",
        ratio:        ["1", "1"],
        threshold:    15940,
        cooldown:     15940,
      }
    }

    await rewardPool.instantiateOrExisting(receipt)
    return rewardPool
  }

}

function getSwapTokens (
  links: Record<string, { address: string, codeHash: string }>,
  admin?: IAgent
): Record<string, SNIP20Contract> {
  const tokens = {}
  for (const [name, {address, codeHash}] of Object.entries(links)) {
    tokens[name] = AMMSNIP20Contract.attach(address, codeHash, admin)
    console.log('getSwapToken', name, address, codeHash)
  }
  return tokens
}

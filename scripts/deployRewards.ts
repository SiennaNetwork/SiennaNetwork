import { randomHex } from '@fadroma/tools'

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

import deployPlaceholderTokens from './deployPlaceholderTokens'

const SIENNA_DECIMALS = 18
const ONE_SIENNA = BigInt(`1${[...Array(SIENNA_DECIMALS)].map(()=>'0').join('')}`)

export type RewardsOptions = {
  chain?:  IChain,
  admin?:  IAgent,
  prefix:  string,
  split?:  number,
  ref?:    string
}

export type RPTRecipient = string
export type RPTAmount    = string
export type RPTConfig = [RPTRecipient, RPTAmount][]

export default async function deployRewards (options: RewardsOptions): RPTConfig {

  const {
    chain = await new Scrt().ready,
    admin = await chain.getAgent(),
    prefix,
    split = 1.0,
    ref   = 'dev'
  } = options

  const
    instance = chain.instances.active,
    SIENNA   = instance.getContract(SiennaSNIP20Contract, 'SiennaSNIP20', admin),
    FACTORY  = instance.getContract(FactoryContract, 'SiennaAMMFactory', admin),
    REWARDS  = new RewardsContract({ prefix, admin, ref })

  const tokens = { SIENNA }
  if (chain.isLocalnet) {
    const placeholderOptions = { chain, admin, prefix, instance }
    Object.assign(tokens, await deployPlaceholderTokens(placeholderOptions))
  } else {
    Object.assign(tokens, getSwapTokens(settings(chain.chainId).swapTokens), admin)
  }

  const rptConfig = []

  rptConfig.push([
    (await deployRewardPool('SIENNA', SIENNA, SIENNA)).address,
    String(BigInt(settings(chain.chainId).rewardPairs.SIENNA) * ONE_SIENNA)
  ])

  const swapPairs = settings(chain.chainId).swapPairs
  if (swapPairs.length > 0) {
    const existingExchanges = await FACTORY.listExchanges()
    const rewards = settings(chain.chainId).rewardPairs
    for (const name of swapPairs) {
      if (rewards && rewards[name]) {
        console.info(`Deploying rewards for ${name}...`)
        //const lpToken = LPTokenContract.attach(lp_token.address, lp_token.code_hash, admin)
        //const reward  = BigInt(rewards[name])
        //const pool    = await deployRewardPool(name, lpToken, SIENNA)
        //rptConfig.push([pool.address, String(reward * ONE_SIENNA)])
      }
    }
  }

  return rptConfig

  async function deployRewardPool (name: string, lpToken: SNIP20Contract, rewardToken: SNIP20Contract) {
    const {codeId, codeHash} = REWARDS
        , options    = { codeId, codeHash, prefix, name, admin, lpToken, rewardToken, ref }
        , rewardPool = new RewardsContract(options)
        , receipt    = instance.contracts[rewardPool.init.label]
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

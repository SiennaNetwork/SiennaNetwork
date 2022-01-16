import { writeFileSync } from 'fs'

import { buildAndUpload } from '@fadroma/ops'
import type { IChain, IAgent } from '@fadroma/ops'
import { Scrt } from '@fadroma/scrt'
import type { SNIP20Contract } from '@fadroma/snip20'
import { bold, randomHex } from '@hackbg/tools'

import settings from '@sienna/settings'

const SIENNA_DECIMALS = 18
const ONE_SIENNA = BigInt(`1${[...Array(SIENNA_DECIMALS)].map(()=>'0').join('')}`)

import {
  AMMContract,
  AMMSNIP20Contract,
  FactoryContract,
  IDOContract,
  LPTokenContract,
  LaunchpadContract,
  RPTContract,
  RewardsContract,
  SiennaSNIP20Contract,
} from '@sienna/api'

export type SwapOptions = {
  chain?:  IChain,
  admin?:  IAgent,
  prefix:  string,
}

export async function deploySwap (options: SwapOptions) {

  const {
    chain = await new Scrt().ready,
    admin = await chain.getAgent(),
    prefix,
  } = options

  const
    instance = chain.instances.active,
    SIENNA   = instance.getContract(SiennaSNIP20Contract, 'SiennaSNIP20', admin),
    RPT      = instance.getContract(RPTContract,          'SiennaRPT',    admin)

  const
    EXCHANGE  = new AMMContract({ prefix, admin }),
    AMMTOKEN  = new AMMSNIP20Contract({ prefix, admin }),
    LPTOKEN   = new LPTokenContract({ prefix, admin }),
    IDO       = new IDOContract({ prefix, admin }),
    LAUNCHPAD = new LaunchpadContract({ prefix, admin }),
    REWARDS   = new RewardsContract({ prefix, admin })

  const
    factoryDeps    = { EXCHANGE, AMMTOKEN, LPTOKEN, IDO, LAUNCHPAD },
    factoryConfig  = settings(chain.chainId).amm,
    factoryOptions = { prefix, admin, config: factoryConfig, ...factoryDeps },
    FACTORY        = new FactoryContract(factoryOptions)

  await buildAndUpload([FACTORY, EXCHANGE, AMMTOKEN, LPTOKEN, IDO, LAUNCHPAD, REWARDS])
  await FACTORY.instantiateOrExisting(instance.contracts['SiennaAMMFactory'])

  /// Obtain a list of token addr/hash pairs for creating liquidity pools

  const tokens = { SIENNA }
  if (chain.isLocalnet) {
    Object.assign(tokens, await deployPlaceholderTokens())
  } else {
    Object.assign(tokens, getSwapTokens(settings(chain.chainId).swapTokens))
  }


  /// Define RPT configuration, starting with single-sided staking


  const rptConfig = [
    [
      (await deployRewardPool('SIENNA', SIENNA, SIENNA)).address,
      String(BigInt(settings(chain.chainId).rewardPairs.SIENNA) * ONE_SIENNA)
    ]
  ]


  /// Create or retrieve liquidity pools,
  /// create their corresponding reward pools,
  /// and add the latter to the RPT configuration.


  const swapPairs = settings(chain.chainId).swapPairs
  if (swapPairs.length > 0) {
    const existingExchanges = await FACTORY.listExchanges()
    const rewards = settings(chain.chainId).rewardPairs
    for (const name of swapPairs) {
      const {lp_token} = await deployLiquidityPool(name, existingExchanges)
      if (rewards && rewards[name]) {
        console.info(`Deploying rewards for ${name}...`)
        const lpToken = LPTokenContract.attach(lp_token.address, lp_token.code_hash, admin)
        const reward  = BigInt(rewards[name])
        const pool    = await deployRewardPool(name, lpToken, SIENNA)
        rptConfig.push([pool.address, String(reward * ONE_SIENNA)])
      }
    }
  }

  if (chain.isMainnet) {
    const rptConfigPath = instance.resolve(`RPTConfig.json`)
    writeFileSync(rptConfigPath, JSON.stringify({config: rptConfig}, null, 2), 'utf8')
    console.info(
      `\n\nWrote ${bold(rptConfigPath)}. `+
      `You should use this file as the basis of a multisig transaction.`
    )
  } else {
    await RPT.configure(rptConfig)
  }

  async function deployLiquidityPool (name: string, existingExchanges: any[]) {
    const [tokenName0, tokenName1] = name.split('-')
    const token0 = tokens[tokenName0]
        , token1 = tokens[tokenName1]
    console.log(`\nLiquidity pool ${bold(name)}...`)
    try {
      const exchange = await FACTORY.getExchange(
        token0.asCustomToken,
        token1.asCustomToken,
        admin
      );
      console.info(`${bold(name)}: Already exists.`)
      return exchange
    } catch (e) {
      if (e.message.includes("Address doesn't exist in storage")) {
        console.info(`${bold(`FACTORY.getExchange(${name})`)}: not found (${e.message}), deploying...`)
        const deployed = await FACTORY.createExchange(
          token0.asCustomToken,
          token1.asCustomToken
        )
        const exchangeReceiptPath = instance.resolve(`SiennaSwap_${name}.json`)
        writeFileSync(exchangeReceiptPath, JSON.stringify(deployed, null, 2), 'utf8')
        console.info(`\nWrote ${bold(exchangeReceiptPath)}.`)
        console.info(bold('Deployed.'), deployed)
        return deployed
      } else {
        throw new Error(`${bold(`FACTORY.getExchange(${name})`)}: not found (${e.message}), deploying...`)
      }
    }
  }

  async function deployRewardPool (name: string, lpToken: SNIP20Contract, rewardToken: SNIP20Contract) {
    const {codeId, codeHash} = REWARDS
        , options    = { codeId, codeHash, prefix, name, admin, lpToken, rewardToken, }
        , rewardPool = new RewardsContract(options)
        , receipt    = instance.contracts[rewardPool.init.label]
    await rewardPool.instantiateOrExisting(receipt)
    return rewardPool
  }


  /// On testnet and mainnet, interoperate with preexisting token contracts.


  function getSwapTokens (links: Record<string, { address: string, codeHash: string }>) {
    const tokens = {}
    for (const [name, {address, codeHash}] of Object.entries(links)) {
      tokens[name] = AMMSNIP20Contract.attach(address, codeHash, admin)
      console.log('getSwapToken', name, address, codeHash)
    }
    return tokens
  }

  /// On localnet, placeholder tokens need to be deployed.

  async function deployPlaceholderTokens () {
    const tokens = {}
    for (
      const [symbol, {label, initMsg}]
      of Object.entries(settings(chain.chainId).placeholderTokens)
    ) {
      const token = tokens[symbol] = new AMMSNIP20Contract({ admin })
      Object.assign(token.blob, { codeId: AMMTOKEN.codeId, codeHash: AMMTOKEN.codeHash })
      Object.assign(token.init, { prefix, label, msg: initMsg })
      Object.assign(token.init.msg, { prng_seed: randomHex(36) })
      const existing = instance.contracts[label]
      await tokens[symbol].instantiateOrExisting(existing)
      await tokens[symbol].setMinters([admin.address], admin)
      await tokens[symbol].mint("100000000000000000000000", admin)
    }
    return tokens
  }

}

import settings from '@sienna/settings'
import type { IChain, IAgent } from '@fadroma/ops'
import { Scrt } from '@fadroma/scrt'
import { bold, randomHex } from '@fadroma/tools'
import { writeFileSync } from 'fs'
import { ONE_SIENNA } from '../ops/index'
import buildAndUpload from './buildAndUpload'

import type { SNIP20Contract as SNIP20 } from '@sienna/api'
import {
  AMMContract,
  AMMSNIP20,
  FactoryContract,
  IDOContract,
  LPToken,
  LaunchpadContract,
  RPTContract,
  RewardsContract,
  SiennaSNIP20,
} from '@sienna/api'

export type SwapOptions = {
  chain?:  IChain,
  admin?:  IAgent,
  prefix:  string,
}

export default async function deploySwap (options: SwapOptions) {

  const {
    chain = await new Scrt().ready,
    admin = await chain.getAgent(),
    prefix,
  } = options

  const
    instance = chain.instances.active,
    SIENNA   = instance.getContract(SiennaSNIP20, 'SiennaSNIP20', admin),
    RPT      = instance.getContract(RPTContract,  'SiennaRPT',    admin)

  const
    EXCHANGE  = new AMMContract({ prefix, admin }),
    AMMTOKEN  = new AMMSNIP20({ prefix, admin }),
    LPTOKEN   = new LPToken({ prefix, admin }),
    IDO       = new IDOContract({ prefix, admin }),
    LAUNCHPAD = new LaunchpadContract({ prefix, admin }),
    REWARDS   = new RewardsContract({ prefix, admin })

  const
    factoryDeps    = { EXCHANGE, AMMTOKEN, LPTOKEN, IDO, LAUNCHPAD },
    factoryConfig  = settings[`amm-${chain.chainId}`],
    factoryOptions = { prefix, admin, config: factoryConfig, ...factoryDeps },
    FACTORY        = new FactoryContract(factoryOptions)

  await buildAndUpload([FACTORY, EXCHANGE, AMMTOKEN, LPTOKEN, IDO, LAUNCHPAD, REWARDS])
  await FACTORY.instantiateOrExisting(instance.contracts['SiennaAMMFactory'])


  /// Obtain a list of token addr/hash pairs for creating liquidity pools


  const tokens = { SIENNA }
  if (chain.isLocalnet) {
    Object.assign(tokens, await deployPlaceholderTokens())
  } else {
    Object.assign(tokens, getSwapTokens(settings[`swapTokens-${chain.chainId}`]))
  }


  /// Define RPT configuration, starting with single-sided staking


  const rptConfig = [
    [
      (await deployRewardPool('SIENNA', SIENNA, SIENNA)).address,
      String(BigInt(settings[`rewardPairs-${chain.chainId}`].SIENNA) * ONE_SIENNA)
    ]
  ]


  /// Create or retrieve liquidity pools,
  /// create their corresponding reward pools,
  /// and add the latter to the RPT configuration.


  const swapPairs = settings[`swapPairs-${chain.chainId}`]
  if (swapPairs.length > 0) {
    const existingExchanges = await FACTORY.listExchanges()
    const rewards = settings[`rewardPairs-${chain.chainId}`]
    for (const name of swapPairs) {
      const [token0, token1] = await deployLiquidityPool(name, existingExchanges)
      if (rewards) {
        const lpToken = await getLPToken(token0, token1)
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

    for (const {pair} of existingExchanges) {
      if ((
        pair.token_0.custom_token.contract_addr === token0.address &&
        pair.token_1.custom_token.contract_addr === token1.address
      ) || (
        pair.token_0.custom_token.contract_addr === token1.address &&
        pair.token_1.custom_token.contract_addr === token0.address
      )) {
        console.info(bold('Already exists.'))
        return [token0, token1]
      }
    }

    const deployed = await FACTORY.createExchange(
      token0.asCustomToken,
      token1.asCustomToken
    )

    const exchangeReceiptPath = instance.resolve(`SiennaSwap_${name}.json`)
    writeFileSync(exchangeReceiptPath, JSON.stringify(deployed, null, 2), 'utf8')
    console.info(`\nWrote ${bold(exchangeReceiptPath)}.`)

    console.info(bold('Deployed.'), deployed)

    return [token0, token1]
  }

  async function getLiquidityPoolInfo (token0: SNIP20, token1: SNIP20) {
    const exchanges = await FACTORY.listExchanges()
    const {address: pairAddress} = exchanges.filter(({pair})=>(
      pair.token_0.custom_token.contract_addr === token0.address &&
      pair.token_1.custom_token.contract_addr === token1.address
    ))[0]
    const {pair_info} = await AMMContract.attach(pairAddress, EXCHANGE.codeHash, admin).pairInfo()
    return pair_info
  }

  async function getLPToken (token0: SNIP20, token1: SNIP20) {
    const {liquidity_token:{address, code_hash}} = await getLiquidityPoolInfo(token0, token1)
    return LPToken.attach(address, code_hash, admin)
  }

  async function deployRewardPool (name: string, lpToken: SNIP20, rewardToken: SNIP20) {
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
      tokens[name] = AMMSNIP20.attach(address, codeHash, admin)
      console.log('getSwapToken', name, address, codeHash)
    }
    return tokens
  }


  /// On localnet, placeholder tokens need to be deployed.


  async function deployPlaceholderTokens () {
    const tokens = {}
    for (
      const [symbol, {label, initMsg}]
      of Object.entries(settings[`placeholderTokens-${chain.chainId}`])
    ) {
      const token = tokens[symbol] = new AMMSNIP20({ admin })
      Object.assign(token.blob, { codeId: AMMTOKEN.codeId, codeHash: AMMTOKEN.codeHash })
      Object.assign(token.init, { prefix, label, msg: initMsg })
      Object.assign(token.init.msg, { prng_seed: randomHex(36) })
      const existing = instance.contracts[label]
      await tokens[symbol].instantiateOrExisting(existing)
    }
    return tokens
  }

}

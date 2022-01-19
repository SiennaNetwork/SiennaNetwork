import { writeFileSync } from 'fs'
import {
  IChain, IAgent, Scrt, buildAndUpload,
  bold, randomHex, timestamp
} from '@fadroma/scrt'
import type { SNIP20Contract } from '@fadroma/snip20'
import settings, { abs } from '@sienna/settings'

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

import { deployPlaceholderTokens } from './deployPlaceholderTokens'
import { getSwapTokens } from './getSwapTokens'
import { deployRewardPool } from './deployRewardPool'
import { deployLiquidityPool } from './deployLiquidityPool'

export async function deploySwap ({
  chain, admin, prefix
}: {
  chain:  IChain,
  admin:  IAgent,
  prefix: string
}) {

  const workspace = abs()

  const deployment = chain.deployments.active

  const SIENNA = deployment.getContract(SiennaSNIP20Contract, 'SiennaSNIP20', admin),
        RPT    = deployment.getContract(RPTContract,          'SiennaRPT',    admin)

  const options = { uploader: admin, instantiator: admin, workspace, chain, prefix, admin }
  const EXCHANGE  = new AMMContract({       ...options }),
        AMMTOKEN  = new AMMSNIP20Contract({ ...options }),
        LPTOKEN   = new LPTokenContract({   ...options }),
        IDO       = new IDOContract({       ...options }),
        LAUNCHPAD = new LaunchpadContract({ ...options }),
        REWARDS   = new RewardsContract({   ...options })

  await buildAndUpload([EXCHANGE, AMMTOKEN, LPTOKEN, IDO, LAUNCHPAD, REWARDS])

  const FACTORY = new FactoryContract({
    ...options,
    exchange_settings: settings(chain.chainId).amm.exchange_settings,
    contracts: {
      snip20_contract:    AMMTOKEN,
      pair_contract:      EXCHANGE,
      lp_token_contract:  LPTOKEN,
      ido_contract:       IDO,
      launchpad_contract: LAUNCHPAD,
    }
  })

  // Deploy the factory
  await buildAndUpload([FACTORY])
  await FACTORY.instantiateOrExisting(deployment.contracts['SiennaAMMFactory'])

  // Obtain a list of token addr/hash pairs for creating liquidity pools
  const tokens: Record<string, SNIP20Contract> = { SIENNA }
  if (chain.isLocalnet) {
    // On localnet, placeholder tokens need to be deployed.
    Object.assign(tokens, await deployPlaceholderTokens({ chain, admin, deployment }))
  } else {
    // On testnet and mainnet, interoperate with preexisting token contracts.
    Object.assign(tokens, getSwapTokens(settings(chain.chainId).swapTokens))
  }

  // Define RPT configuration,
  // starting with single-sided staking
  const sssss = await deployRewardPool({
    chain, admin, deployment,
    REWARDS,
    suffix: 'SIENNA',
    lpToken: SIENNA,
    rewardToken: SIENNA,
  })
  const rptConfig = [
    [
      sssss.address,
      String(BigInt(settings(chain.chainId).rewardPairs.SIENNA) * ONE_SIENNA)
    ]
  ]

  // Create or retrieve liquidity pools,
  // create their corresponding reward pools,
  // and add the latter to the RPT configuration.
  const swapPairs = settings(chain.chainId).swapPairs
  if (swapPairs.length > 0) {
    const existingExchanges = await FACTORY.listExchanges()
    const rewards = settings(chain.chainId).rewardPairs
    const liquidityPoolOptions = {
      admin,
      FACTORY,
      existingExchanges,
      tokens,
      deployment
    }
    for (const name of swapPairs) {
      const {lp_token} = await deployLiquidityPool({
        ...liquidityPoolOptions,
        name
      })
      if (rewards && rewards[name]) {
        console.info(`Deploying rewards for ${name}...`)
        const reward = String(BigInt(rewards[name]) * ONE_SIENNA)
        const pool = await deployRewardPool({
          chain, admin, deployment,
          REWARDS,
          suffix: name,
          lpToken: new LPTokenContract({
            address:  lp_token.address,
            codeHash: lp_token.code_hash,
            admin
          }),
          rewardToken: SIENNA,
        })
        rptConfig.push([pool.address, reward])
      }
    }
  }

  if (chain.isMainnet) {
    const rptConfigPath = deployment.resolve(`RPTConfig.json`)
    writeFileSync(rptConfigPath, JSON.stringify({config: rptConfig}, null, 2), 'utf8')
    console.info(
      `\n\nWrote ${bold(rptConfigPath)}. `+
      `You should use this file as the basis of a multisig transaction.`
    )
  } else {
    await RPT.tx(admin).configure(rptConfig)
  }

}

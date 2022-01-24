import {
  Migration,
  IChain, IAgent,
  bold, randomHex, timestamp,
  writeFileSync
} from '@hackbg/fadroma'
import type { SNIP20Contract } from '@fadroma/snip20'
import settings from '@sienna/settings'

const SIENNA_DECIMALS = 18
const ONE_SIENNA = BigInt(`1${[...Array(SIENNA_DECIMALS)].map(()=>'0').join('')}`)

import {
  FactoryContract,

  AMMContract,
  AMMSNIP20Contract,
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

export async function deploySwap (migration: Migration) {

  const {
    workspace,
    chain,
    admin,
    prefix
  } = migration

  const deployment = chain.deployments.active

  const SIENNA = deployment.getContract(SiennaSNIP20Contract, 'SiennaSNIP20', admin),
        RPT    = deployment.getContract(RPTContract,          'SiennaRPT',    admin)

  const EXCHANGE  = new AMMContract({ ...migration })
  await chain.buildAndUpload([EXCHANGE])

  const AMMTOKEN  = new AMMSNIP20Contract({ ...migration })
  const LPTOKEN   = new LPTokenContract({   ...migration })
  const IDO       = new IDOContract({       ...migration })
  const LAUNCHPAD = new LaunchpadContract({ ...migration })
  await buildAndUpload([AMMTOKEN, LPTOKEN, IDO, LAUNCHPAD])

  const FACTORY = new FactoryContract({
    ...migration,
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
  await FACTORY.instantiateOrExisting(deployment.contracts['SiennaAMMFactory'])

  // Obtain a list of token addr/hash pairs for creating liquidity pools
  const tokens: Record<string, SNIP20Contract> = { SIENNA }
  if (chain.isLocalnet) {
    // On localnet, placeholder tokens need to be deployed.
    Object.assign(tokens, await deployPlaceholderTokens({ ...migration }))
  } else {
    // On testnet and mainnet, interoperate with preexisting token contracts.
    Object.assign(tokens, getSwapTokens(settings(chain.chainId).swapTokens))
  }

  // Deploy pools and add them to the RPT configuration.

  // 1. Stake SIENNA to earn SIENNA
  const singleSidedStaking = await deployRewardPool({
    ...migration,
    suffix:     'SIENNA',
    lpToken:     SIENNA,
    rewardToken: SIENNA,
  })

  // 2. Add that to the RPT config
  const rptConfig = [
    [
      singleSidedStaking.address,
      String(BigInt(settings(chain.chainId).rewardPairs.SIENNA) * ONE_SIENNA)
    ]
  ]

  // 3. If there are any initial swap pairs defined
  const swapPairs = settings(chain.chainId).swapPairs
  if (swapPairs.length > 0) {

    const existingExchanges = await FACTORY.listExchanges()
    const rewardPairs = settings(chain.chainId).rewardPairs
    const liquidityPoolOptions = {
      admin,
      FACTORY,
      existingExchanges,
      tokens,
      deployment
    }

    for (const name of swapPairs) {

      // 4. Instantiate each one in the factory,
      //    keeping the handle to the LP token
      const {lp_token} = await run(deployLiquidityPool, {
        ...liquidityPoolOptions,
        name
      })

      // 5. If this swap pair has an assigned reward pool in the config
      if (rewardPairs && rewardPairs[name]) {

        console.info(`Deploying rewards for ${name}...`)

        const reward = String(BigInt(rewardPairs[name]) * ONE_SIENNA)

        // 6. Stake LP to earn sienna. 
        const pool = await run(deployRewardPool, {
          suffix: name,
          rewardToken: SIENNA,
          lpToken: new LPTokenContract({
            address:  lp_token.address,
            codeHash: lp_token.code_hash,
            admin
          }),
        })

        // 7. Add that to the RPT config
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

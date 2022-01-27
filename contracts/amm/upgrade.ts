import process from 'process'
import { Console, MigrationContext, bold, colors, timestamp, writeFileSync } from '@hackbg/fadroma'
import type { SNIP20Contract } from '@fadroma/snip20'
import {
  RPTContract,
  FactoryContract, ExchangeInfo,
  AMMContract,
  LPTokenContract,
  RewardsContract
} from '@sienna/api'
import settings, { workspace } from '@sienna/settings'
import { deployRewardPool } from './deploy'

type MultisigTX = any

const console    = Object.assign(Console('@sienna/amm/upgrade'), { table: global.console.table })
const pick       = (...keys) => x => keys.reduce((y, key)=>{y[key]=x[key];return y}, {})
const essentials = pick('codeId', 'codeHash', 'address', 'label')

export async function upgradeFactoryAndRewards ({
  timestamp, chain, admin, deployment, prefix, run,
  RPT              = deployment.getContract(RPTContract,      'SiennaAMMFactory', admin),
  OLD_FACTORY      = deployment.getContract(FactoryContract,  'SiennaAMMFactory', admin),
  OLD_REWARD_POOLS = deployment.getContracts(RewardsContract, 'SiennaRewards',    admin)
}: MigrationContext & {
  // The arbiter of it all.
  // Will redirect funds in a 30:70 proportion
  // from old version to new version.
  RPT?:             RPTContract,
  // The v1 factory. We'll terminate this now,
  // so that new pairs cannot be created from the v1 factory.
  OLD_FACTORY?:      FactoryContract
  // The reward pools attached to some of the LP tokens
  // of the liquidity pools of the v1 factory.
  // We'll disincentivise those in RPT now,
  // and eventually terminate them in the next migration.
  OLD_REWARD_POOLS?: RewardsContract[]
}): Promise<MultisigTX[]> {
  // The liquidity pools of the v1 factory.
  // We'll disincentivise those in RPT now,
  // and eventually terminate them in the next migration.
  const OLD_EXCHANGES: ExchangeInfo[] = await OLD_FACTORY.exchanges
  // The LP tokens of the liquidity pools of the v1 factory.
  // We'll disincentivise those in RPT now,
  // and eventually terminate them in the next migration.
  // Let's report some initial status.
  console.log()
  console.info(bold('Current factory:'))
  console.table(essentials(OLD_FACTORY))

  console.log()
  console.info(bold("Current factory's exchanges"), "(to be disincentivised):")
  for (const {
    name,
    EXCHANGE: { codeId, codeHash, address },
    TOKEN_0,
    TOKEN_1,
    LP_TOKEN
  } of OLD_EXCHANGES) {

    console.info(
      ' ',
      bold(colors.inverse(name)).padEnd(30), // wat
      `(code id ${bold(String(codeId))})`.padEnd(34),
      bold(address)
    )

    await printToken(TOKEN_0)
    await printToken(TOKEN_1)
    await printToken(LP_TOKEN)

    async function printToken (TOKEN) {
      if (typeof TOKEN === 'string') {
        console.info(
          `   `,
          bold(TOKEN.padEnd(10))
        )
      } else {
        const {name, symbol} = await TOKEN.q(admin).tokenInfo()
        console.info(
          `   `,
          bold(symbol.padEnd(10)),
          name.padEnd(25).slice(0, 25),
          TOKEN.address
        )
      }
    }

  }

  process.exit(123)
  console.info(bold("Current factory's exchanges' LP tokens"), "(to be disincentivised):")
  console.table(OLD_LP_TOKENS.map(essentials))
  console.info(bold("V2 reward pools attached to current factory's LP tokens"), "(to be disincentivised)")
  console.table(OLD_REWARD_POOLS.map(essentials))
  // The new contracts.
  // Their addresses should be added to the frontend.
  const NEW_FACTORY: FactoryContract = new FactoryContract({
    workspace,
    prefix,
    suffix: `@v2.0.0+${timestamp}`,
    admin,
    exchange_settings: settings(chain.chainId).amm.exchange_settings,
  })
  console.info(
    bold('Deploying new factory'), NEW_FACTORY.label
  )
  const contracts = await OLD_FACTORY.getContracts()
  NEW_FACTORY.setContracts(contracts)
  await chain.buildAndUpload([NEW_FACTORY])
  await NEW_FACTORY.instantiate()
  console.info(
    bold('Deployed new factory'), NEW_FACTORY.address
  )
  console.table(essentials(NEW_FACTORY))
  // The new liquidity pools.
  // Their addresses should be added to the frontend.
  const NEW_EXCHANGES: AMMContract[]     = []
  const NEW_LP_TOKENS: LPTokenContract[] = []
  for (const { address, token_0, token_1 } of OLD_EXCHANGES) {
    console.info(
      bold('Upgrading exchange'), address
    )
    const { EXCHANGE, LP_TOKEN } = await NEW_FACTORY.createExchange(token_0, token_1)
    NEW_EXCHANGES.push(EXCHANGE)
    NEW_LP_TOKENS.push(LP_TOKEN)
  }
  console.info(bold("Newly created exchanges from V2 factory:"))
  console.table(NEW_EXCHANGES.map(essentials))
  console.info(bold("And their new LP tokens:"))
  console.table(NEW_LP_TOKENS.map(essentials))
  // The v3 reward pools.
  // Their addresses should be added to the frontend.
  const NEW_REWARD_POOLS: RewardsContract[] = []
  //for (const LP_TOKEN of NEW_LP_TOKENS) {
    //const { REWARDS } = await run(deployRewardPool, {
    //})
    //NEW_REWARD_POOLS.push()
  //}
  console.table(NEW_REWARD_POOLS.map(essentials))
  return []
}

export async function replaceRewardPool ({
  chain,
  admin,
  prefix,
  deployment,
  rewardPoolLabel
}: MigrationContext & {
  rewardPoolLabel: string
}) {
  console.info(
    `Upgrading reward pool ${bold(rewardPoolLabel)}` +
    `\nin deployment ${bold(prefix)}` +
    `\non ${bold(chain.chainId)}` +
    `\nas ${bold(admin.address)}\n`
  )
  // This is the old reward pool
  const POOL = deployment.getContract(RewardsContract, rewardPoolLabel, admin)
  // Find address of pool in RPT config
  const RPT  = deployment.getContract(RPTContract, 'SiennaRPT', admin)
  const {config} = await RPT.status
  let found: number = NaN
  for (let i = 0; i < config.length; i++) {
    console.info(config[i])
    if (config[i][0] === POOL.address) {
      if (!isNaN(found)) {
        console.info(`Address ${bold(POOL.address)} found in RPT config twice.`)
        process.exit(1)
      }
      found = i
    }
  }
  if (isNaN(found)) {
    console.info(`Reward pool ${bold(POOL.address)} not found in RPT ${bold(RPT.address)}`)
    process.exit(1)
  }
  console.info(`Replacing reward pool ${POOL.address}`)
  const [
    LP_TOKEN,
    REWARD_TOKEN
  ] = await Promise.all([
    POOL.lpToken(),
    POOL.rewardToken()
  ])
  const NEW_POOL = new RewardsContract({
    prefix,
    name:   `${rewardPoolLabel}`,
    suffix: `@${timestamp()}`
    admin,
    lpToken:     LP_TOKEN,
    rewardToken: REWARD_TOKEN
  })
  await chain.buildAndUpload([NEW_POOL])
  await NEW_POOL.instantiate()
  config[found][0] = NEW_POOL.address
  if (chain.isMainnet) {
    const rptConfigPath = deployment.resolve(`RPTConfig.json`)
    writeFileSync(rptConfigPath, JSON.stringify({config}, null, 2), 'utf8')
    console.info(
      `\n\nWrote ${bold(rptConfigPath)}. `+
      `You should use this file as the basis of a multisig transaction.`
    )
  } else {
    await RPT.tx().configure(config)
  }
  await POOL.tx().close(`Moved to ${NEW_POOL.address}`)
}

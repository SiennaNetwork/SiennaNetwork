import type { IChain, IAgent } from '@fadroma/ops'
import { bold, timestamp } from '@fadroma/tools'
import process from 'process'
import { writeFileSync } from 'fs'
import buildAndUpload from './buildAndUpload'

export default async function replaceRewardPool (
  chain: IChain,
  admin: IAgent,
  label: string
) {

  const { name: prefix, getContract } = chain.instances.active

  console.log(
    `Upgrading reward pool ${bold(label)}` +
    `\nin deployment ${bold(prefix)}` +
    `\non ${bold(chain.chainId)}` +
    `\nas ${bold(admin.address)}\n`
  )

  const OLD_POOL = getContract(RewardsContract, label, admin)
      , RPT      = getContract(RPTContract, 'SiennaRPT', admin)

  const {config} = await RPT.status
  let found: number = NaN
  for (let i = 0; i < config.length; i++) {
    console.log(config[i])
    if (config[i][0] === OLD_POOL.address) {
      if (!isNaN(found)) {
        console.log(`Address ${bold(OLD_POOL.address)} found in RPT config twice.`)
        process.exit(1)
      }
      found = i
    }
  }

  if (isNaN(found)) {
    console.log(`Reward pool ${bold(OLD_POOL.address)} not found in RPT ${bold(RPT.address)}`)
    process.exit(1)
  }

  console.log(`Replacing reward pool ${OLD_POOL.address}`)

  const [LP_TOKEN, REWARD_TOKEN] = await Promise.all([
    OLD_POOL.getLPToken(admin),
    OLD_POOL.getRewardToken(admin)
  ])

  const NEW_POOL = new RewardsContract({
    prefix, label: `${label}@${timestamp()}`, admin,
    lpToken:     LP_TOKEN,
    rewardToken: REWARD_TOKEN
  })
  await buildAndUpload([NEW_POOL])
  await NEW_POOL.instantiate()

  config[found][0] = NEW_POOL.address

  if (chain.isMainnet) {
    const rptConfigPath = instance.resolve(`RPTConfig.json`)
    writeFileSync(rptConfigPath, JSON.stringify({config}, null, 2), 'utf8')
    console.info(
      `\n\nWrote ${bold(rptConfigPath)}. `+
      `You should use this file as the basis of a multisig transaction.`
    )
  } else {
    await RPT.configure(config)
  }

  await OLD_POOL.close(`Moved to ${NEW_POOL.address}`)
}

export function printRewardsContracts (chain: IChain) {
  if (chain && chain.instances.active) {
    const {name, contracts} = chain.instances.active
    const isRewardPool = (x: string) => x.startsWith('SiennaRewards_')
    const rewardsContracts = Object.keys(contracts).filter(isRewardPool)
    if (rewardsContracts.length > 0) {
      console.log(`\nRewards contracts in ${bold(name)}:`)
      for (const name of rewardsContracts) {
        console.log(`  ${colors.green('✓')}  ${name}`)
      }
    } else {
      console.log(`\nNo rewards contracts.`)
    }
  } else {
    console.log(`\nSelect an instance to pick a reward contract.`)
  }
}
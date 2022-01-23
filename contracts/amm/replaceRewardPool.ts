import type { Migration } from '@fadroma/scrt'
import { buildAndUpload } from '@fadroma/scrt'
import { bold, colors, timestamp } from '@hackbg/tools'
import { RewardsContract, RPTContract } from '@sienna/api'
import process from 'process'
import { writeFileSync } from 'fs'

export async function replaceRewardPool (options: Migration & { rewardPoolLabel: string }) {

  const {
    resolve,
    chain,
    admin,
    prefix,
    getContract,
    rewardPoolLabel
  } = options

  console.log(
    `Upgrading reward pool ${bold(rewardPoolLabel)}` +
    `\nin deployment ${bold(prefix)}` +
    `\non ${bold(chain.chainId)}` +
    `\nas ${bold(admin.address)}\n`
  )

  // This is the old reward pool
  const POOL = getContract(RewardsContract, rewardPoolLabel, admin)

  // Find address of pool in RPT config
  const RPT  = getContract(RPTContract, 'SiennaRPT', admin)
  const {config} = await RPT.status
  let found: number = NaN
  for (let i = 0; i < config.length; i++) {
    console.log(config[i])
    if (config[i][0] === POOL.address) {
      if (!isNaN(found)) {
        console.log(`Address ${bold(POOL.address)} found in RPT config twice.`)
        process.exit(1)
      }
      found = i
    }
  }
  if (isNaN(found)) {
    console.log(`Reward pool ${bold(POOL.address)} not found in RPT ${bold(RPT.address)}`)
    process.exit(1)
  }

  console.log(`Replacing reward pool ${POOL.address}`)

  const [
    LP_TOKEN,
    REWARD_TOKEN
  ] = await Promise.all([
    POOL.lpToken(),
    POOL.rewardToken()
  ])

  const NEW_POOL = new RewardsContract({
    prefix,
    label: `${rewardPoolLabel}@${timestamp()}`,
    admin,
    lpToken:     LP_TOKEN,
    rewardToken: REWARD_TOKEN
  })
  await buildAndUpload([NEW_POOL])
  await NEW_POOL.instantiate()

  config[found][0] = NEW_POOL.address

  if (chain.isMainnet) {
    const rptConfigPath = resolve(`RPTConfig.json`)
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

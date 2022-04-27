import { MigrationContext, Template, buildAndUpload, bold, randomHex } from '@hackbg/fadroma'
import * as API from '@sienna/api'
import getSettings, { ONE_SIENNA } from '@sienna/settings'
import { linkStruct } from '../misc'
import { adjustRPTConfig } from '../Configure'
import { versions, contracts, source } from '../Build'

export interface RewardsDeployOptions extends MigrationContext {
  /** Which address will be admin
    * of the new reward pools.
    * Defaults to the executing agent. */
  admin:       string,
  /** The reward token.
    * Defaults to SIENNA */
  reward:      API.Snip20Client,
  /** Version of the reward pools to deploy. */
  version:     API.RewardsAPIVersion,
  /** The AMM version to which
    * the rewards will be attached. */
  ammVersion:  API.AMMVersion,
  /** Whether the new reward pools
    * should be configured in the RPT */
  adjustRPT:   boolean

  settings: {
    /** List of reward pairs to generate. */
    rewardPairs: Record<string, any>,
    timekeeper: string
  }
}

type RewardsDeployResult = API.RewardsClient[]

async function deployRewards (context: RewardsDeployOptions): Promise<RewardsDeployResult> {

  const {
    run,

    version  = 'v3' as API.RewardsAPIVersion,
    ref      = versions.Rewards[version]
    builder,
    uploader,
    src      = source('sienna-rewards', ref)
    template = await buildAndUpload(builder, uploader, src),

    deployAgent, deployment,

    agent,
    admin = agent.address,
    settings: { rewardPairs, timekeeper } = getSettings(agent.chain.mode),

    clientAgent,
    reward = new API.Snip20Client({...deployment.get('SIENNA'), agent: clientAgent}),

    ammVersion = { v3: 'v2', v2: 'v1' }[version] as API.AMMVersion,
    adjustRPT = true

  } = context

  const createPools = Object.entries(rewardPairs)

  const results = await deployment.initMany(deployAgent, template, createPools.map(([name, amount])=>{
    // get the staked token by name
    if (name !== 'SIENNA') name = `AMM[${ammVersion}].${name}.LP`
    const staked = new API.Snip20Client(deployment.get(name))
    // define the name of the reward pool from the staked token
    name = `${name}.Rewards[${version}]`
    return [name, makeRewardsInitMsg[version](reward, staked, admin, timekeeper)]
  }))

  const rptConfig = createPools.map(([name, amount], i)=>[results[i].address, String(BigInt(amount)*ONE_SIENNA)])

  const clients = results.map(result=>new API.RewardsClient[version]({...result, agent: clientAgent}))

  if (adjustRPT) {
    await run(adjustRPTConfig, { RPT_CONFIG: rptConfig })
  }

  return clients

}

/** Rewards v2 and v3 have different APIs,
  * including different init message formats: */
const makeRewardsInitMsg = {

  "v2" (reward, staked, admin) {

    let threshold = 15940
    let cooldown  = 15940

    const { SIENNA_REWARDS_V2_BONDING } = process.env
    if (SIENNA_REWARDS_V2_BONDING) {
      console.warn(
        bold('Environment override'),
        'SIENNA_REWARDS_V2_BONDING=',
        SIENNA_REWARDS_V2_BONDING
      )
      threshold = Number(SIENNA_REWARDS_V2_BONDING)
      cooldown  = Number(SIENNA_REWARDS_V2_BONDING)
    }

    return {
      admin,
      lp_token:     linkStruct(staked),
      reward_token: linkStruct(reward),
      viewing_key:  randomHex(36),
      ratio:        ["1", "1"],
      threshold,
      cooldown
    }

  },

  "v3" (reward, staked, admin, timekeeper) {

    let bonding = 86400

    const { SIENNA_REWARDS_V3_BONDING } = process.env
    if (SIENNA_REWARDS_V3_BONDING) {
      console.warn(
        bold('Environment override'),
        'SIENNA_REWARDS_V3_BONDING=',
        SIENNA_REWARDS_V3_BONDING
      )
      bonding = Number(SIENNA_REWARDS_V3_BONDING)
    }

    return {
      admin,
      config: {
        reward_vk:    randomHex(36),
        lp_token:     linkStruct(staked),
        reward_token: linkStruct(reward),
        timekeeper,
        bonding,
      }
    }

  }

}

export interface RewardsUpgradeOptions extends MigrationContext {
  settings: {
    /** Which address will be admin
      * of the new reward pools.
      * Defaults to the executing agent. */
    admin:         string
    /** Which address can call BeginEpoch
      * on the new reward pools.
      * Defaults to the value of `admin` */
    timekeeper:    string
  }
  /** The reward token.
    * Defaults to SIENNA */
  reward:        API.Snip20Client
  /** Old version that we are migrating from. */
  vOld:    API.RewardsAPIVersion
  /** New version that we are migrating to. */
  vNew:    API.RewardsAPIVersion
  /** Code id and code hash of new version. */
  template:      Template
  /** Version of the AMM that the
    * new reward pools will attach to. */
  newAmmVersion: API.AMMVersion
}

export interface RewardsUpgradeResult {
  REWARD_POOLS: API.RewardsClient[]
}

async function upgradeRewards (context: RewardsUpgradeOptions): Promise<RewardsUpgradeResult> {
  const {
    run,

    vOld       = 'v2',
    vNew       = 'v3',
    ref        = versions.Rewards[vNew],
    src        = source('sienna-rewards', ref),
    builder,
    uploader,
    template   = await buildAndUpload(builder, uploader, src)
    newAmmVersion = 'v2'

    deployAgent, deployment, prefix, timestamp,
    settings: {
      admin      = deployAgent.address,
      timekeeper = admin
    } = getSettings(deployAgent.chain.mode),

    clientAgent,
    reward = new API.Snip20Client({ ...deployment.get('SIENNA'), agent: clientAgent }),
  } = context

  // establish client classes
  const OldRewardsClient = API.RewardsClient[vOld]
  const NewRewardsClient = API.RewardsClient[vNew]

  // Collect info about old reward pools
  // (namely, what are their staked tokens)
  const isOldPool   = name => name.endsWith(`.Rewards[${vOld}]`)
  const oldNames    = Object.keys(deployment.receipts).filter(isOldPool)
  const oldReceipts = oldNames.map(name=>deployment.get(name))
  const oldPools    = oldReceipts.map(r=>new OldRewardsClient({...r, agent: clientAgent}))
  const stakedTokens     = new Map()
  const stakedTokenNames = new Map()
  await Promise.all(oldPools.map(async pool=>{
    console.info(bold('Getting staked token info for:'), pool.name)
    if (pool.name === 'SIENNA.Rewards[v2]') {
      stakedTokens.set(pool, reward)
      stakedTokenNames.set(reward, 'SIENNA')
    } else {
      const staked = await pool.getStakedToken()
      stakedTokens.set(pool, staked)
      const name = await staked.getPairName()
      stakedTokenNames.set(staked, name)
    }
  }))

  // Create new reward pools with same staked tokens as old reward pools
  // WARNING: This might've been the cause of the wrong behavior
  //          of the AMM+Rewards migration; new pools should point to new LP tokens.
  const newPools = await deployment.initMany(deployAgent, template, oldPools.map(oldPool=>{
    const staked = stakedTokens.get(oldPool)
    const name = (staked.address === deployment.get('SIENNA').address)
      ? `SIENNA.Rewards[${vNew}]`
      : `AMM[${newAmmVersion}].${stakedTokenNames.get(staked)}.LP.Rewards[${vNew}]`
    return [name, makeRewardsInitMsg[vNew](reward, staked, admin, timekeeper)]
  }))
  console.log({newPools})

  // Return clients to new reward pools.
  const newPoolClients = newPools.map(r=>new NewRewardsClient({...r, agent: clientAgent}))
  return { REWARD_POOLS: newPoolClients }

}

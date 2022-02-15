import {
  Console, bold,
  Scrt_1_2, Snip20Contract,
  randomHex, timestamp,
  readFileSync
} from "@hackbg/fadroma"

const console = Console('@sienna/rewards/Contract')

import { RewardsAPIVersion, RewardsClient } from './RewardsClient'
export * from './RewardsClient'

import { Agent, MigrationContext, Template, Snip20Client } from '@hackbg/fadroma'
import { LPTokenContract, LPTokenClient } from '@sienna/lp-token'
import { RPTContract, RPTClient, RPTConfig } from '@sienna/rpt'
import { SiennaSnip20Contract } from '@sienna/snip20-sienna'
import { AMMVersion } from '@sienna/exchange'
import getSettings, { workspace, ONE_SIENNA } from '@sienna/settings'
import { Init } from './schema/init.d'

const {
  SIENNA_REWARDS_V2_BONDING,
  SIENNA_REWARDS_V3_BONDING
} = process.env

const makeRewardInitMsg = {

  "v2" (admin, staked, reward) {
    let threshold = 15940
    let cooldown  = 15940
    if (SIENNA_REWARDS_V2_BONDING) {
      console.warn(bold('Environment override'), 'SIENNA_REWARDS_V2_BONDING=', SIENNA_REWARDS_V2_BONDING)
      threshold = Number(SIENNA_REWARDS_V2_BONDING)
      cooldown  = Number(SIENNA_REWARDS_V2_BONDING)
    }
    return {
      admin,
      lp_token:     { address: staked?.address, code_hash: staked?.codeHash },
      reward_token: { address: reward?.address, code_hash: reward?.codeHash },
      viewing_key:  randomHex(36),
      ratio:        ["1", "1"],
      threshold,
      cooldown
    }
  },

  "v3" (admin, timekeeper, staked, reward) {
    let bonding = 86400
    if (SIENNA_REWARDS_V3_BONDING) {
      console.warn(bold('Environment override'), 'SIENNA_REWARDS_V3_BONDING=', SIENNA_REWARDS_V3_BONDING)
      bonding = Number(SIENNA_REWARDS_V3_BONDING)
    }
    return {
      admin,
      config: {
        reward_vk:    randomHex(36),
        lp_token:     { address: staked?.address, code_hash: staked?.codeHash },
        reward_token: { address: reward?.address, code_hash: reward?.codeHash },
        timekeeper,
        bonding,
      }
    }
  }

}

export abstract class RewardsContract extends Scrt_1_2.Contract<RewardsClient> {

  name   = 'Rewards'
  source = { workspace, crate: 'sienna-rewards' }

  abstract Client
  abstract version: RewardsAPIVersion

  static "v2" = class RewardsContract_v2 extends RewardsContract {

    version = "v2" as RewardsAPIVersion
    name    = `Rewards[${this.version}]`
    source = { workspace, crate: 'sienna-rewards', ref: 'rewards-2.1.2' }

    initMsg?: any // TODO v2 init type
    Client = RewardsClient['v2']

    constructor ({ template, admin, staked, reward }: {
      template?: Template,
      admin?:    string,
      staked?:   Snip20Client,
      reward?:   Snip20Client,
    } = {}) {
      super()
      if (template) this.template = template
      this.initMsg = makeRewardInitMsg['v2'](admin, staked, reward)
    }

    /** Command. Deploy legacy Rewards v2. */
    static deploy = function deployRewards_v2 ({run}) {
      return run(RewardsContract.deployRewards, { version: 'v2', adjustRPT: true })
    }

    /** Command. Replace v2 reward pools with v3. */
    static upgrade = {
      "v3": function upgradeRewards_v2_to_v3 (input) {
        return RewardsContract.upgradeRewards({ ...input, oldVersion: 'v2', newVersion: 'v3', })
      }
    }
  }

  static "v3" = class RewardsContract_v3 extends RewardsContract {

    version = "v3" as RewardsAPIVersion
    name    = `Rewards[${this.version}]`
    source  = { workspace, crate: 'sienna-rewards', ref: '39e87e4' }

    initMsg?: Init
    Client = RewardsClient['v3']

    constructor ({ template, admin, timekeeper, staked, reward }: {
      template?:   Template,
      admin?:      string,
      timekeeper?: string,
      staked?:     Snip20Client,
      reward?:     Snip20Client,
    } = {}) {
      super()
      if (template) this.template = template
      this.initMsg = makeRewardInitMsg['v3'](admin, timekeeper, staked, reward)
    }

    /** Command. Deploy Rewards v3. */
    static deploy = async function deployRewards_v3 ({run}) {
      const { RPT_CONFIG, REWARD_POOLS } = await run(RewardsContract.deployRewards, { version: 'v3' })
      return await run(RPTContract.adjustConfig, { RPT_CONFIG })
    }

    /** Command. The v3 to v3 upgrade tests user migration. */
    static upgrade = {
      "v3": function upgradeRewards_v2_to_v3 (input) {
        return RewardsContract.upgradeRewards({ ...input, oldVersion: 'v3', newVersion: 'v3', })
      }
    }

    /** Command. Import addresses from a bundle that initialized multiple
      * rewards contracts, and query their configuration. */
    static importReceipts = async function importRewardsReceipts ({
      agent,
      deployment
    }) {
      const bundleReceiptPath = agent.chain.stateRoot.resolve('rewards-v3.json')
      const bundleReceiptData = JSON.parse(readFileSync(bundleReceiptPath, 'utf8'))
      const addresses = bundleReceiptData.logs.map(({ msg_index, log, events: [ message, wasm ] })=>{
        const address = message.attributes[4].value
        console.log(address)
        return address
      })
      const stakedTokens = new Map()
      const stakedTokenNames = new Map()
      const { codeId, codeHash } = agent.chain.uploads.load('sienna-rewards@39e87e4.wasm')
      await Promise.all(addresses.map(async address=>{
        const client = new RewardsClient.v3({ address, codeHash, agent })
        const label = await client.label
        deployment.add(label.split('/')[1], {
          label,
          codeId,
          codeHash,
          address,
          initTx: bundleReceiptData.txhash
        })
      }))
    }
  }

  static "v2+v3" = {
    /** Command. Deploy both versions simultaneously,
      * splitting the balance evenly in the RPT config. */
    deploy: () => { throw new Error('deprecated') }
  }

  /** Command. Attach a specified version of Sienna Rewards
    * to a specified version of Sienna Swap. */
  static deployRewards = deployRewards

  static upgradeRewards = upgradeRewards

}

async function deployRewards (context: MigrationContext & {
  /** Which address will be admin of the new reward pools.
    * Defaults to the executing agent. */
  admin:       string,
  /** The reward token. Defaults to SIENNA */
  reward:      Snip20Client,
  /** Version of the rewards that we will be deploying */
  version:     RewardsAPIVersion,
  /** Uploaded codeId+codeHash for rewards[version]. */
  template:    Template,
  /** The AMM version to which to attach the rewards. */
  ammVersion:  AMMVersion,
  /** List of reward pairs to generate. */
  rewardPairs: any,
  /** Prevent label clashes when iterating locally. */
  suffix:      string
  /** Whether the new reward pools should be configured in the RPT */
  adjustRPT:   boolean
}): Promise<[RewardsClient, string][]> {
  const {
    deployment, agent, run, suffix,
    uploadAgent,
    deployAgent,
    clientAgent,

    admin       = agent.address,
    reward      = new Snip20Client({ ...deployment.get('SIENNA'), agent: clientAgent }),
    version     = 'v3' as RewardsAPIVersion,
    template    = (await agent.buildAndUpload([new RewardsContract[version]({})]))[0],
    ammVersion  = { v3: 'v2', v2: 'v1' }[version] as AMMVersion,
    rewardPairs = getSettings(agent.chain.id).rewardPairs,
    adjustRPT   = true
  } = context

  const rewardPoolsToCreate = []

  for (let [name, budgetAllocation] of Object.entries(rewardPairs)) {
    if (name !== 'SIENNA') name = `AMM[${ammVersion}].${name}.LP`
    const staked = new Snip20Client(deployment.get(name))
    name = `${name}.Rewards[${version}]`
    const contract = new RewardsContract[version]({ template, admin, staked, reward })
    rewardPoolsToCreate.push([
      [contract, contract.initMsg, name],   // init options
      String(BigInt(budgetAllocation as string) * ONE_SIENNA) // budget allocation
    ])
  }

  const getInitOptions = x=>x[0]
  await deployment.instantiate(deployAgent, ...rewardPoolsToCreate.map(getInitOptions))

  if (adjustRPT) {
    const getAddressAndBudget = x=>[x[0][0].address, x[1]]
    await run(RPTContract.adjustConfig, {
      RPT_CONFIG: rewardPoolsToCreate.map(getAddressAndBudget)
    })
  }

  const getClientAndBudget = x=>x[0][0].client(clientAgent)
  return rewardPoolsToCreate.map(getClientAndBudget) as [RewardsClient, string][]
}

async function upgradeRewards (context: MigrationContext & {
  /** Which address will be admin of the new reward pools.
    * Defaults to the executing agent. */
  admin:         string
  /** Which address can call BeginEpoch on the new reward pools.
    * Defaults to the value of `admin` */
  timekeeper:    string
  /** The reward token.
    * Defaults to SIENNA */
  reward:        Snip20Client
  /** Old version that we are migrating from. */
  oldVersion:    RewardsAPIVersion
  /** New version that we are migrating to. */
  newVersion:    RewardsAPIVersion
  /** Code id and code hash of new version. */
  template:      Template
  /** Version of the AMM that the new reward pools will attach to. */
  newAmmVersion: AMMVersion
}): Promise<{
  REWARD_POOLS: RewardsClient[]
}> {
  const {
    timestamp, agent, deployment, prefix, run, suffix = `+${timestamp}`,
    uploadAgent,
    deployAgent,
    clientAgent,

    admin      = getSettings(agent.chain.id).admin      || agent.address,
    timekeeper = getSettings(agent.chain.id).timekeeper || admin,
    reward = new Snip20Client({ ...deployment.get('SIENNA'), agent }),
    oldVersion,
    newVersion,
    template = (await uploadAgent.buildAndUpload([new RewardsContract[newVersion]({})]))[0],
    newAmmVersion = 'v2'
  } = context

  const isOldRewardPool =
    name => name.endsWith(`.Rewards[${oldVersion}]`)
  const oldRewardPoolNames =
    Object.keys(deployment.receipts).filter(isOldRewardPool)
  const OldRewardsClient =
    RewardsClient[oldVersion]
  const oldRewardPools =
    oldRewardPoolNames.map(name=>new OldRewardsClient({ ...deployment.receipts[name], agent }))

  console.log({oldRewardPoolNames})

  const stakedTokens = new Map()
  const stakedTokenNames = new Map()
  await Promise.all(oldRewardPools.map(async pool=>{
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

  const NewRewardsContract = RewardsContract[newVersion]
  const newRewardPools = []
  for (const oldRewardPool of oldRewardPools) {
    const staked = stakedTokens.get(oldRewardPool)
    const newRewardPool = new NewRewardsContract({
      template,
      admin,
      timekeeper,
      staked,
      reward
    })
    let name
    if (staked.address === deployment.get('SIENNA').address) {
      name = `SIENNA.Rewards[${newVersion}]`
    } else {
      name = `AMM[${newAmmVersion}].${stakedTokenNames.get(staked)}.LP.Rewards[${newVersion}]`
    }
    newRewardPools.push([newRewardPool, newRewardPool.initMsg, name])
  }

  await deployment.instantiate(deployAgent, ...newRewardPools)

  return { REWARD_POOLS: newRewardPools }

}

async function rerouteRewardFunding () { /* TODO */ }

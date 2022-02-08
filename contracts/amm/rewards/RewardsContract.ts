import { Console, bold, Scrt_1_2, Snip20Contract, randomHex, timestamp } from "@hackbg/fadroma"

const console = Console('@sienna/rewards/Contract')

import { RewardsAPIVersion, RewardsClient } from './RewardsClient'
export * from './RewardsClient'

import { MigrationContext, Template, Snip20Client } from '@hackbg/fadroma'
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

  "v3" (admin, staked, reward) {
    let bonding = 86400
    if (SIENNA_REWARDS_V3_BONDING) {
      console.warn(bold('Environment override'), 'SIENNA_REWARDS_V3_BONDING=', SIENNA_REWARDS_V3_BONDING)
      bonding = Number(SIENNA_REWARDS_V3_BONDING)
    }
    return {
      admin,
      config: {
        reward_vk:    randomHex(36),
        timekeeper:   admin,
        lp_token:     { address: staked?.address, code_hash: staked?.codeHash },
        reward_token: { address: reward?.address, code_hash: reward?.codeHash },
        bonding,
      }
    }
  }

}

export abstract class RewardsContract extends Scrt_1_2.Contract<RewardsClient> {

  name   = 'Rewards'
  source = { workspace, crate: 'sienna-rewards' }
  abstract Client
  abstract version:        RewardsAPIVersion

  static "v2" = class RewardsContract_v2 extends RewardsContract {

    name    = `Rewards[${this.version}]`
    source = { workspace, crate: 'sienna-rewards', ref: 'rewards-2.1.2' }
    version = "v2" as RewardsAPIVersion
    initMsg?: any // TODO v2 init type
    Client = RewardsClient['v2']

    constructor ({ template, admin, staked, reward }: {
      template?: Template,
      admin:     string,
      staked:    Snip20Client,
      reward:    Snip20Client,
    }) {
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
    initMsg?: Init
    Client = RewardsClient['v3']

    constructor ({ template, admin, staked, reward }: {
      template: Template,
      admin:    string,
      staked:   Snip20Client,
      reward:   Snip20Client,
    }) {
      super()
      if (template) this.template = template
      this.initMsg = makeRewardInitMsg['v3'](admin, staked, reward)
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
  }

  static "v2+v3" = {
    /** Command. Deploy both versions simultaneously,
      * splitting the balance evenly in the RPT config. */
    deploy: deployRewards_v2_and_v3
  }

  /** Command. Attach a specified version of Sienna Rewards
    * to a specified version of Sienna Swap. */
  static deployRewards = deployRewards

  static upgradeRewards = upgradeRewards

}

async function deployRewards ({

  deployment, agent, run,

  SIENNA      = new Snip20Client({ ...deployment.get('SIENNA'), agent }),
  version     = 'v3' as RewardsAPIVersion,
  ammVersion  = { v3: 'v2', v2: 'v1' }[version] as AMMVersion,
  rewardPairs = getSettings(agent.chain.id).rewardPairs,
  suffix      = `+${timestamp()}`,

  adjustRPT = true

}: MigrationContext & {

  SIENNA:      Snip20Client,
  version:     RewardsAPIVersion,
  ammVersion:  AMMVersion,
  rewardPairs: any,
  suffix:      string

  /** Whether the new reward pools should be configured in the RPT */
  adjustRPT: boolean

}): Promise<[RewardsClient, string][]> {

  const contract = new RewardsContract[version]({})
  await agent.chain.buildAndUpload(agent, [contract])
  const { template } = contract
  const rewardPoolsToCreate = []
  const admin = agent.address
  const reward = SIENNA

  for (let [name, budgetAllocation] of Object.entries(rewardPairs)) {
    if (name !== 'SIENNA') name = `AMM[${ammVersion}].${name}.LP`
    const staked = new Snip20Client(deployment.get(name))
    name = `${name}.Rewards[${version}]`
    const contract = new RewardsContract[version]({ template, admin, staked, reward })
    rewardPoolsToCreate.push([
      [contract, contract.initMsg, name],   // init options
      String(BigInt(budgetAllocation) * ONE_SIENNA) // budget allocation
    ])
  }

  const getInitOptions = x=>x[0]
  await deployment.instantiate(agent, ...rewardPoolsToCreate.map(getInitOptions))

  if (adjustRPT) {
    const getAddressAndBudget = x=>[x[0][0].address, x[1]]
    await run(RPTContract.adjustConfig, { RPT_CONFIG: rewardPoolsToCreate.map(getAddressAndBudget) })
  }

  const getClientAndBudget = x=>x[0][0].client(agent)
  return rewardPoolsToCreate.map(getClientAndBudget) as [RewardsClient, string][]

}

type MultisigTX = any
const pick       = (...keys) => x => keys.reduce((y, key)=>{y[key]=x[key];return y}, {})
const essentials = pick('codeId', 'codeHash', 'address', 'label')

async function deployRewards_v2_and_v3 ({run}) {
  const [V2, V3] = await Promise.all([
    run(RewardsContract.deployRewards, { version: 'v2' }),
    run(RewardsContract.deployRewards, { version: 'v3' })
  ])
  const REWARD_POOLS = [ ...V2.REWARD_POOLS, ...V3.REWARD_POOLS ]
  console.table(REWARD_POOLS.reduce(
    (table, {label, address, codeId, codeHash})=>
      Object.assign(table, {
        [label]: { address: address, codeId: codeId, codeHash: codeHash }
      }), {}))
  const RPT_CONFIG  = [ ...V2.RPT_CONFIG,   ...V3.RPT_CONFIG   ]
  return await run(RPTContract.adjustConfig, { RPT_CONFIG })
  return { RPT_CONFIG, REWARD_POOLS }
}

async function upgradeRewards ({
  timestamp, agent, deployment, prefix, run,
  oldVersion,
  newVersion,
  OldRewardsContract = RewardsContract[oldVersion],
  NewRewardsContract = RewardsContract[newVersion],
  SIENNA = new Snip20Client({ ...deployment.get('SIENNA'), agent }),
  RPT    = new RPTClient({    ...deployment.get('RPT'),    agent }),
  REWARD_POOLS = deployment.getAll(
    `Rewards[${oldVersion}].`, name => new OldRewardsContract({agent})
  ),
  version,
  suffix = `+${timestamp}`
}: MigrationContext & {
  oldVersion:         RewardsAPIVersion,
  newVersion:         RewardsAPIVersion,
  OldRewardsContract: new(input:any)=>RewardsContract,
  NewRewardsContract: new(input:any)=>RewardsContract,
  SIENNA: Snip20Client,
  RPT:    RPTClient
  REWARD_POOLS: RewardsClient[]
}): Promise<{
  REWARD_POOLS: RewardsClient[]
}> {
  const NEW_REWARD_POOLS: RewardsContract[] = []
  for (const REWARDS of REWARD_POOLS) {
    //console.log({REWARDS})
    //console.log(REWARDS.lpToken())
    //process.exit(123)
    const LP_TOKEN = REWARDS.lpToken
    const {symbol} = await LP_TOKEN.info
    let name
    if (symbol === 'SIENNA') {
      name = 'SIENNA'
    } else {
      const [LP, TOKEN0, TOKEN1] = (await LP_TOKEN.friendlyName).split('-')
      name = `AMM[v2].${TOKEN0}-${TOKEN1}.LP`
    }
    //console.log()
    //console.info(bold('Upgrading reward pool'), name)
    const { REWARDS: NEW_REWARDS } = await run(deployRewardPool, {
      version,
      name,
      lpToken: LP_TOKEN,
      rewardToken: SIENNA
    })
    NEW_REWARD_POOLS.push(NEW_REWARDS)
  }
  const count = bold(String(NEW_REWARD_POOLS.length))
  console.info(`Deployed`, count, version, `reward pools.`)
  return {
    REWARD_POOLS: NEW_REWARD_POOLS.map(contract=>contract.client(agent))
  }
}

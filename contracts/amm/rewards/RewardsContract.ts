import { Console, bold, Scrt_1_2, SNIP20Contract, ContractConstructor, randomHex, } from "@hackbg/fadroma"

const console = Console('@sienna/rewards/Contract')

import getSettings, { ONE_SIENNA, workspace } from '@sienna/settings'
import { SiennaSNIP20Contract } from '@sienna/snip20-sienna'
import { LPTokenContract } from '@sienna/lp-token'
import { RPTContract } from '@sienna/rpt'

import { Init } from './schema/init.d'
export * from './RewardsApi'
import {
  RewardsTransactions_v2,
  RewardsTransactions_v3,
  RewardsQueries_v2,
  RewardsQueries_v3,
  RewardsAPIVersion
} from './RewardsApi'

const {
  SIENNA_REWARDS_V2_BONDING,
  SIENNA_REWARDS_V3_BONDING
} = process.env

export abstract class RewardsContract<T, Q> extends Scrt_1_2.Contract<T, Q> {

  workspace = workspace
  name  = this.name  || 'Rewards'
  crate = this.crate || 'sienna-rewards'
  abstract version: RewardsAPIVersion

  /** Instance representing the reward token. */
  abstract rewardToken (): Promise<SNIP20Contract>
  abstract lpToken     (): Promise<SNIP20Contract>

  static "v2" = class RewardsContract_v2 extends RewardsContract<
    RewardsTransactions_v2,
    RewardsQueries_v2
  > {
    version = "v2" as RewardsAPIVersion
    name    = `Rewards[${this.version}]`
    ref     = "rewards-2.1.2"
    initMsg?: any // TODO v2 init type
    Transactions = RewardsTransactions_v2
    Queries      = RewardsQueries_v2
    constructor (input) {
      super(input)
      const { lpToken, rewardToken, agent } = input
      if (SIENNA_REWARDS_V2_BONDING) {
        console.warn('Env var SIENNA_REWARDS_V2_BONDING is set to', SIENNA_REWARDS_V2_BONDING)
      }
      this.initMsg = {
        admin:        agent?.address,
        lp_token:     lpToken?.link,
        reward_token: rewardToken?.link,
        viewing_key:  "",
        ratio:        ["1", "1"],
        threshold:    Number(SIENNA_REWARDS_V2_BONDING||15940),
        cooldown:     Number(SIENNA_REWARDS_V2_BONDING||15940),
      }
    }

    async lpToken <T extends SNIP20Contract> (T = LPTokenContract): Promise<T> {
      const at = Math.floor(+new Date()/1000)
      const {pool_info} = await this.query({pool_info:{at}})
      const {address, code_hash} = pool_info.lp_token
      return new T({ address, codeHash: code_hash, agent: this.agent }) as T
    }
    async rewardToken <T extends SNIP20Contract> (T = SNIP20Contract): Promise<T> {
      throw new Error('not implemented')
    }

    /** Command. Deploy legacy Rewards v2. */
    static deploy = async function deployRewards_v2 ({run}) {
      const { RPT_CONFIG, REWARD_POOLS } =
        await run(RewardsContract.deployAll, { version: 'v2' })
      await run(RPTContract.adjustConfig, { RPT_CONFIG })
      return { RPT_CONFIG, REWARD_POOLS }
    }

    /** Command. Replace v2 reward pools with v3. */
    static upgrade = {
      "v3": function upgradeRewards_v2_to_v3 (input) {
        return RewardsContract.upgradeRewards({
          ...input,
          oldVersion: 'v2', OldRewardsContract: RewardsContract.v3,
          newVersion: 'v3', NewRewardsContract: RewardsContract.v3,
        })
      }
    }
  }

  static "v3" = class RewardsContract_v3 extends RewardsContract<
    RewardsTransactions_v3,
    RewardsQueries_v3
  > {
    version = "v3" as RewardsAPIVersion
    name    = `Rewards[${this.version}]`
    initMsg?: Init
    Transactions = RewardsTransactions_v3
    Queries      = RewardsQueries_v3
    constructor (input) {
      super(input)
      const { lpToken, rewardToken, agent } = input
      if (SIENNA_REWARDS_V3_BONDING) {
        console.warn('Env var SIENNA_REWARDS_V3_BONDING is set', SIENNA_REWARDS_V3_BONDING)
      }
      this.initMsg = {
        admin: agent?.address,
        config: {
          reward_vk:    randomHex(36),
          bonding:      Number(process.env.SIENNA_REWARDS_V3_BONDING||86400),
          timekeeper:   agent?.address,
          lp_token:     lpToken?.link,
          reward_token: rewardToken?.link,
        }
      }
    }

    async lpToken <T extends SNIP20Contract> (T = LPTokenContract): Promise<T> {
      const { lp_token: { address, code_hash } } = await this.q().config()
      return new T({ address, codeHash: code_hash, agent: this.agent }) as T
    }
    async rewardToken <T extends SNIP20Contract> (T = SNIP20Contract): Promise<T> {
      throw new Error('not implemented')
    }
    get epoch (): Promise<number> {
      return this.q().pool_info().then(pool_info=>pool_info.clock.number)
    }

    /** Command. Deploy Rewards v3. */
    static deploy = async function deployRewards_v3 ({run}) {
      const { RPT_CONFIG, REWARD_POOLS } = await run(RewardsContract.deployAll, { version: 'v3' })
      return await run(RPTContract.adjustConfig, { RPT_CONFIG })
    }

    /** Command. v3 to v3 upgrade tests user migration. */
    static upgrade = {
      "v3": function upgradeRewards_v2_to_v3 (input) {
        return RewardsContract.upgradeRewards({
          ...input,
          oldVersion: 'v3', OldRewardsContract: RewardsContract.v3,
          newVersion: 'v3', NewRewardsContract: RewardsContract.v3,
        })
      }
    }
  }

  static "v2+v3" = {
    /** Command. Deploy both versions simultaneously,
      * splitting the balance evenly in the RPT config. */
    deploy: async function deployRewards_v2_and_v3 ({run}) {
      const [V2, V3] = await Promise.all([
        run(RewardsContract.deployAll, { version: 'v2' }),
        run(RewardsContract.deployAll, { version: 'v3' })
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
  }

  /** Deploy a single reward pool. Primitive of both deploySSSSS and deployRewardPools */
  static deployOne = async function deployRewardPool ({
    agent, chain, deployment, prefix,
    lpToken,
    rewardToken = deployment.getThe('SIENNA', new SiennaSNIP20Contract({ agent })),
    name        = 'UNKNOWN',
    version     = 'v3',
  }) {
    name = `Rewards[${version}].${name}`
    const REWARDS = new RewardsContract[version]({ lpToken, rewardToken, agent })
    await chain.buildAndUpload(agent, [REWARDS])
    REWARDS.name = name
    await deployment.getOrInit(agent, REWARDS)
    return { REWARDS }
  }

  /** Command. Attach a spcified version of Sienna Rewards
    * to a specified version of Sienna Swap. */
  static deployAll = async function deployAll ({
    deployment, agent, run,
    SIENNA     = deployment.getThe('SIENNA', new SiennaSNIP20Contract({ agent })),
    version    = 'v3',
    ammVersion = {v3:'v2',v2:'v1'}[version],
  }) {
    const { SSSSS_POOL, RPT_CONFIG_SSSSS } =
      await run(deploySSSSS, { SIENNA, version })
    const { REWARD_POOLS, RPT_CONFIG_SWAP_REWARDS } =
      await run(deployRewardPools, { SIENNA, version, ammVersion })
    return {
      REWARD_POOLS: [ SSSSS_POOL, ...REWARD_POOLS ],
      RPT_CONFIG:   [ ...RPT_CONFIG_SSSSS, ...RPT_CONFIG_SWAP_REWARDS ]
    }
  }

  static async upgradeRewards ({
    timestamp, chain, agent, deployment, prefix, run,
    oldVersion, OldRewardsContract,
    newVersion, NewRewardsContract,
    SIENNA = deployment.getThe('SIENNA', new SiennaSNIP20Contract({agent})),
    RPT    = deployment.getThe('RPT',    new RPTContract({agent})),
    REWARD_POOLS = deployment.getAll('Rewards[v2].', name => new OldRewardsContract({agent})),
    version,
    suffix = `+${timestamp}`
  }) {
    const NEW_REWARD_POOLS: RewardsContract[] = []
    for (const REWARDS of REWARD_POOLS) {
      //console.log({REWARDS})
      //console.log(REWARDS.lpToken())
      //process.exit(123)
      const LP_TOKEN = REWARDS.lpToken
      const {symbol} = await LP_TOKEN.info
      let name
      if (symbol === 'SIENNA') {
        name = 'SSSSS'
      } else {
        const [LP, TOKEN0, TOKEN1] = (await LP_TOKEN.friendlyName).split('-')
        name = `${TOKEN0}-${TOKEN1}`
      }
      console.log()
      console.info(bold('Upgrading reward pool'), name)
      const options = {
        version, name, suffix,
        lpToken: LP_TOKEN, rewardToken: SIENNA
      }
      NEW_REWARD_POOLS.push((await run(RewardsContract.deployOne, options)).REWARDS)
    }
    console.info(`Deployed`, bold(String(NEW_REWARD_POOLS.length)), version, `reward pools.`)
    return { REWARD_POOLS: NEW_REWARD_POOLS }
  }

}

type MultisigTX = any
const pick       = (...keys) => x => keys.reduce((y, key)=>{y[key]=x[key];return y}, {})
const essentials = pick('codeId', 'codeHash', 'address', 'label')

/** Deploy SIENNA/SIENNA SINGLE-SIDED STAKING,
  * where you stake SIENNA to earn SIENNA. */
export async function deploySSSSS ({
  run, chain, deployment, agent,
  SIENNA  = deployment.getThe('SIENNA', new SiennaSNIP20Contract({ agent })),
  version = 'v3',
  settings: { rewardPairs } = getSettings(chain.id),
}) {
  if (!rewardPairs || rewardPairs.length === 1) {
    throw new Error(`@sienna/rewards: needs rewardPairs setting for ${chain.id}`)
  }
  const name        = 'SSSSS'
  const lpToken     = SIENNA
  const rewardToken = SIENNA
  const { REWARDS } = await run(RewardsContract.deployOne, { version, name, lpToken: SIENNA })
  return {
    SSSSS_POOL: REWARDS, RPT_CONFIG_SSSSS: [
      [
        REWARDS.address,
        String(BigInt(getSettings(chain.id).rewardPairs.SIENNA) * ONE_SIENNA)
      ]
    ]
  }
}

/** Deploy the rest of the reward pools, where you stake a LP token to earn SIENNA. */
export async function deployRewardPools ({
  chain, agent, deployment, prefix, run,
  SIENNA                    = deployment.getThe('SIENNA', new SiennaSNIP20Contract({ agent })),
  version                   = 'v3',
  ammVersion                = {v3:'v2',v2:'v1'}[version],
  settings: { rewardPairs } = getSettings(chain.id),
  REWARD_POOLS              = [],
  split                     = 1.0,
  RPT_CONFIG_SWAP_REWARDS   = [],
}) {
  if (!rewardPairs || rewardPairs.length === 1) {
    throw new Error(`@sienna/rewards: needs rewardPairs setting for ${chain.id}`)
  }
  for (let [name, reward] of Object.entries(rewardPairs)) {
    // ignore SSSSS pool - that is deployed separately
    if (name === 'SIENNA') continue
    // find LP token to attach to
    const lpTokenName = `AMM[${ammVersion}].${name}.LP`
    const lpToken = deployment.getThe(lpTokenName, new LPTokenContract({ agent }))
    // create a reward pool
    const options = { version, name, lpToken }
    console.info('Deploying', bold(name), version, 'for', bold(lpTokenName))
    const { REWARDS } = await run(RewardsContract.deployOne, options)
    REWARD_POOLS.push(REWARDS)
    // collect the RPT budget line
    const reward = BigInt(rewardPairs[name]) / BigInt(1 / split)
    const budget = [REWARDS.address, String(reward * ONE_SIENNA)]
    RPT_CONFIG_SWAP_REWARDS.push(budget)
  }
  return { REWARD_POOLS, RPT_CONFIG_SWAP_REWARDS }
}

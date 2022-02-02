import { Scrt_1_2, SNIP20Contract, ContractConstructor, randomHex, bold, Console } from "@hackbg/fadroma"

const console = Console('@sienna/rewards/Contract')

import { SiennaSNIP20Contract } from '@sienna/snip20-sienna'
import { AMMFactoryContract } from '@sienna/factory'
import { AMMExchangeContract, ExchangeInfo } from '@sienna/exchange'
import { LPTokenContract } from '@sienna/lp-token'
import { RPTContract, RPTConfig } from '@sienna/rpt'
import getSettings, { workspace, SIENNA_DECIMALS, ONE_SIENNA } from '@sienna/settings'

import { Init } from './schema/init.d'
export * from './RewardsApi'
import { RewardsTransactions, RewardsQueries, RewardsAPIVersion } from './RewardsApi'

export abstract class RewardsContract extends Scrt_1_2.Contract<RewardsTransactions, RewardsQueries> {

  workspace = workspace
  name  = this.name  || 'Rewards'
  crate = this.crate || 'sienna-rewards'
  abstract version: RewardsAPIVersion

  RewardTokenContract: ContractConstructor<SNIP20Contract> = SiennaSNIP20Contract
  abstract rewardToken <T extends SNIP20Contract> (
    Contract: ContractConstructor<SNIP20Contract>
  ): Promise<T>

  LPTokenContract: ContractConstructor<SNIP20Contract> = LPTokenContract
  abstract lpToken <T extends SNIP20Contract> (
    Contract: ContractConstructor<SNIP20Contract>
  ): Promise<T>

  static v2 = class RewardsContract_v2 extends RewardsContract {
    version = 'v2' as RewardsAPIVersion
    name    = `Rewards[${this.version}]`
    ref     = 'rewards-2.1.2'
    initMsg?: any // TODO v2 init type
    Transactions = RewardsTransactions // TODO v2 executors
    Queries      = RewardsQueries
    constructor (input) {
      super(input)
      const { lpToken, rewardToken, agent } = input
      this.initMsg = {
        admin:        agent?.address,
        lp_token:     lpToken?.link,
        reward_token: rewardToken?.link,
        viewing_key:  "",
        ratio:        ["1", "1"],
        threshold:    15940,
        cooldown:     15940,
      }
    }
    async lpToken <T extends SNIP20Contract> (T = this.LPTokenContract): Promise<T> {
      const at = Math.floor(+new Date()/1000)
      const {pool_info} = await this.query({pool_info:{at}})
      const {address, code_hash} = pool_info.lp_token
      return new T({ address, codeHash: code_hash, agent: this.agent }) as T
    }
    async rewardToken <T extends SNIP20Contract> (T = this.LPTokenContract): Promise<T> {
      throw new Error('not implemented')
    }
  }

  static v3 = class RewardsContract_v3 extends RewardsContract {
    version = 'v3' as RewardsAPIVersion
    name    = `Rewards[${this.version}]`
    initMsg?: Init
    Transactions = RewardsTransactions
    Queries      = RewardsQueries
    constructor (input) {
      super(input)
      const { lpToken, rewardToken, agent } = input
      this.initMsg = {
        admin: agent?.address,
        config: {
          reward_vk:    randomHex(36),
          bonding:      86400,
          timekeeper:   agent?.address,
          lp_token:     lpToken?.link,
          reward_token: rewardToken?.link,
        }
      }
    }
    async lpToken (SNIP20 = this.LPTokenContract): Promise<any> {
      throw new Error('v3 does not expose config')
    }
    async rewardToken (SNIP20 = this.RewardTokenContract): Promise<any> {
      throw new Error('v3 does not expose config')
    }
    get epoch (): Promise<number> {
      return this.q().pool_info().then(pool_info=>pool_info.clock.number)
    }
  }

}

import type { IAgent, IContract, ContractState, ContractConstructor } from "@fadroma/scrt"
import { ScrtContract_1_2 } from "@fadroma/scrt"
import { randomHex } from '@hackbg/tools'
import { SNIP20Contract } from '@fadroma/snip20'
import { LPTokenContract } from '@sienna/lp-token'
import { workspace } from '@sienna/settings'
import { Init } from './schema/init.d'

export class RewardsContract extends ScrtContract_1_2 {

  crate = 'sienna-rewards'

  name = 'SiennaRewards'

  initMsg: Init = {
    admin: this.instantiator.address,
    config: {}
  }

  constructor (options: ContractState & {
    /** Admin agent */
    admin:       IAgent,
    /** Address of other user that can increment the epoch */
    timekeeper:  string,
    /** Staked token */
    lpToken:     LPTokenContract,
    /** Rewarded token */
    rewardToken: SNIP20Contract,
    /** Bonding period config */
    bonding?:     number,
  }) {
    super(options)
    this.initMsg.admin = options.admin.address
    this.initMsg.config = {
      reward_vk:    randomHex(36),
      bonding:      options.bonding || 86400,
      timekeeper:   options.timekeeper,
      lp_token:     options.lpToken?.link,
      reward_token: options.rewardToken?.link,
    }
  }

  Q (agent: IAgent = this.instantiator) {

    const query = (method: string, args: any) =>
      agent.query(this.link, method, args)

    return {

      async poolInfo () {
        const at = Math.floor(+ new Date() / 1000)
        return await query("rewards", { pool_info: { at } })
      },

      async getEpoch () {
        const info = await this.poolInfo()
        return info.rewards.pool_info.clock.number
      },

      async getRewardToken (TOKEN: ContractConstructor) {
        const { address, code_hash } = (await this.poolInfo(agent)).reward_token
        return new TOKEN({ address, codeHash: code_hash, admin: agent })
      },

      async getLPToken (TOKEN: ContractConstructor) {
        const { address, code_hash } = (await this.poolInfo(agent)).lp_token
        return new TOKEN({ address, codeHash: code_hash, admin: agent })
      }

    }

  }

  TX (agent: IAgent = this.instantiator) {

    const execute = (method: string, args: any) =>
      agent.execute(this.link, method, args)

    return {
      setLPToken (address: string, code_hash: string) {
        return execute('rewards', { configure: { lp_token: { address, code_hash } } })
      },
      deposit (amount: string) {
        return execute('rewards', { deposit: { amount } })
      },
      withdraw (amount: string) {
        return execute('rewards', { withdraw: { amount } })
      },
      claim () {
        return execute('rewards', { claim: {} })
      },
      close (message: string) {
        return execute('rewards', { close: { message } })
      },
      beginEpoch (next_epoch: number) {
        return execute('rewards', { begin_epoch: { next_epoch } })
      },
      drain (snip20: Link, recipient: string, key?: string) {
        return execute('drain', { snip20, recipient, key })
      },
      enableMigrationFrom (link: Link) {
        return execute('immigration', { enable_migration_from: link })
      },
      disableMigrationFrom (link: Link) {
        return execute('immigration', { disable_migration_from: link })
      },
      requestMigration (link: Link) {
        return execute('immigration', { request_migration: link })
      },
      enableMigrationTo (link: Link) {
        return execute('emigration', { enable_migration_to: link })
      },
      disableMigrationTo (link: Link) {
        return execute('emigration', { disable_migration_to: link })
      },
    }

  }

}

type Link = { address: string, code_hash: string }

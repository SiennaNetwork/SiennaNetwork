import { resolve, dirname } from 'path'
import { fileURLToPath } from 'url'
const workspace = resolve(dirname(fileURLToPath(import.meta.url)), '../..')

import type { IAgent } from "@fadroma/scrt"
import { ScrtContract } from "@fadroma/scrt"
import { randomHex } from '@fadroma/tools'
import { SNIP20Contract } from '@fadroma/snip20'
import { LPTokenContract } from '@sienna/lp-token'

export type RewardsOptions = {
  codeId?:      number
  codeHash?:    string

  prefix?:      string
  name?:        string
  label?:       string

  admin?:       IAgent
  timekeeper?:  string

  lpToken?:     SNIP20Contract
  rewardToken?: SNIP20Contract

  bonding?:     number
}

export class RewardsContract extends ScrtContract {

  static schema = ScrtContract.loadSchemas(
    import.meta.url, {
      initMsg:     "./schema/init.json",
      queryMsg:    "./schema/query.json",
      queryAnswer: "./schema/response.json",
      handleMsg:   "./schema/handle.json",
    })

  static attach (
    address:  string,
    codeHash: string,
    agent:    IAgent
  ) {
    const instance = new RewardsContract({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }

  constructor ({
    codeId, codeHash,
    prefix, label, name,
    admin, timekeeper,
    lpToken, rewardToken,
    bonding = 86400,
  }: RewardsOptions = {}) {

    super({
      agent:  admin,
      schema: RewardsContract.schema,
      prefix,
      label:  label || `SiennaRewards_${name}_Pool`
    })

    if (codeId) {
      this.blob.codeId = codeId
    }

    if (codeHash) {
      this.blob.codeHash = codeHash
    }

    Object.assign(this.init.msg, {
      admin: admin?.address
    })

    const reward_vk = randomHex(36)

    Object.defineProperty(this.init.msg, 'config', {
      enumerable: true,
      get () {
        return {
          lp_token:     lpToken?.link,
          reward_token: rewardToken?.link,
          reward_vk,
          bonding,
          timekeeper
        }
      }
    })

  }

  code = {
    ...this.code,
    workspace,
    crate: 'sienna-rewards'
  }

  init = {
    ...this.init,
    label: this.init.label||'SiennaRewards',
    msg: {
      admin: this.instantiator,
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

      async getRewardToken (TOKEN: { attach: Function } = SNIP20) {
        const { address, code_hash } = (await this.poolInfo(agent)).reward_token
        return TOKEN.attach(address, code_hash, agent)
      },

      async getLPToken (TOKEN: { attach: Function } = LPTokenContract) {
        const { address, code_hash } = (await this.poolInfo(agent)).lp_token
        return TOKEN.attach(address, code_hash, agent)
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

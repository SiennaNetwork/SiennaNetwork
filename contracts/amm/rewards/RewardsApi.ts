import { Console, bold } from '@hackbg/fadroma'
import { QueryExecutor, TransactionExecutor } from '@hackbg/fadroma'

const console = Console('@sienna/rewards/Api')

export type RewardsAPIVersion = 'v2'|'v3'

export class RewardsQueries_v2 extends QueryExecutor {
  async pool_info (
    at = Math.floor(+ new Date() / 1000)
  ) {
    const result = await this.query({ pool_info: { at } })
    return result.pool_info
  }
  async user_info (
    key     = "",
    address = this.agent.address,
    at,
  ) {
    at = at || (await this.agent.block).header.height
    const result = await this.query({user_info: { address, key, at } })
    return result.user_info
  }
}

export class RewardsTransactions_v2 extends TransactionExecutor {
  lock (amount: string) {
    return this.execute({ lock: { amount } })
  }
  claim () {
    return this.execute({ claim: {} })
  }
  set_viewing_key(key: string) {
    return this.execute({ set_viewing_key: { key } })
  }
}

export class RewardsQueries_v3 extends QueryExecutor {
  async config () {
    const result = await this.query({ rewards: "config" })
    return result.rewards.config
  }
  async pool_info (
    at = Math.floor(+ new Date() / 1000)
  ) {
    const result = await this.query({ rewards: { pool_info: { at } } })
    return result.rewards.pool_info
  }
  async user_info (
    key     = "",
    address = this.agent.address,
    at      = Math.floor(+ new Date() / 1000)
  ) {
    const result = await this.query({ rewards: { user_info: { address, key, at } } })
    return result.rewards.user_info
  }
}

export class RewardsTransactions_v3 extends TransactionExecutor {
  async epoch () {
    const { pool_info: { clock: { number } } } = await this.contract.q(this.agent).pool_info()
    return this.execute({ rewards: { begin_epoch: number + 1 } })
  }

  setLPToken (address: string, code_hash: string) {
    return this.execute({
      rewards: { configure: { lp_token: { address, code_hash } } }
    })
  }
  lock (amount: string) {
    console.warn(
      '[@sienna/rewards] Deprecation warning: v2 Lock has been renamed to Deposit in v3. ' +
      'It will be gone in 3.1 - plan accordingly.'
    )
    return this.deposit(amount)
  }
  deposit (amount: string) {
    return this.execute({
      rewards: { deposit: { amount } }
    })
  }
  claim () {
    return this.execute({
      rewards: { claim: {} }
    })
  }
  close (message: string) {
    return this.execute({
      rewards: { close: { message } }
    })
  }
  withdraw (amount: string) {
    return this.execute({ rewards: { withdraw: { amount } } })
  }
  beginEpoch (next_epoch: number) {
    return this.execute({ rewards: { begin_epoch: { next_epoch } } })
  }
  drain (snip20: Link, recipient: string, key?: string) {
    return this.execute({ drain: { snip20, recipient, key } })
  }
  enableMigrationTo (link: Link) {
    return this.execute({ emigration:  { enable_migration_to:    link } })
  }
  disableMigrationTo (link: Link) {
    return this.execute({ emigration:  { disable_migration_to:   link } })
  }
  enableMigrationFrom (link: Link) {
    return this.execute({ immigration: { enable_migration_from:  link } })
  }
  disableMigrationFrom (link: Link) {
    return this.execute({ immigration: { disable_migration_from: link } })
  }
  requestMigration (link: Link) {
    return this.execute({ immigration: { request_migration:      link } })
  }
  set_viewing_key(key: string) {
    return this.execute({ set_viewing_key: { key } })
  }
}

type Link = { address: string, code_hash: string }

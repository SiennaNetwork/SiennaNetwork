import { TransactionExecutor } from '@hackbg/fadroma'

type Link = { address: string, code_hash: string }

export class RewardsTransactions extends TransactionExecutor {

  setLPToken (address: string, code_hash: string) {
    return this.execute({ rewards: { configure:   { lp_token: { address, code_hash } } } })
  }

  deposit (amount: string) {
    return this.execute({ rewards: { deposit:     { amount } } })
  }

  withdraw (amount: string) {
    return this.execute({ rewards: { withdraw:    { amount } } })
  }

  claim () {
    return this.execute({ rewards: { claim:       {} } })
  }

  close (message: string) {
    return this.execute({ rewards: { close:       { message } } })
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
  };
  enableMigrationFrom (link: Link) {
    return this.execute({ immigration: { enable_migration_from:  link } })
  }
  disableMigrationFrom (link: Link) {
    return this.execute({ immigration: { disable_migration_from: link } })
  }
  requestMigration (link: Link) {
    return this.execute({ immigration: { request_migration:      link } })
  }
}


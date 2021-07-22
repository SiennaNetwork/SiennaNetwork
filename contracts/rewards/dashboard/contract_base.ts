import { UIContext } from './widgets'

export class Pool {
    ui: UIContext
    constructor (ui: UIContext) {
      this.ui = ui
    }
}

export class User {
    ui: UIContext
    pool: Pool
    name: string
    balance: number
    constructor (ui: UIContext, pool: Pool, name: string, balance: number) {
      this.ui      = ui
      this.pool    = pool
      this.name    = name
      this.balance = balance
    }
}

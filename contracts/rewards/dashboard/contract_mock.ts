import { COLORS } from './helpers'
import { UIContext, Table, Log, PieChart, StackedPieChart } from './widgets'
import { Pool, User } from './contract_base'

// settings ----------------------------------------------------------------------------------------
const TIME_SCALE          = 30
const COOLDOWN            = 17280/TIME_SCALE
const FUND_INTERVAL       = 17280/TIME_SCALE
const THRESHOLD           = 17280/TIME_SCALE
const USER_GIVES_UP_AFTER = Infinity
const MAX_USERS           = 100
const MAX_INITIAL         = 1000

// root of time ------------------------------------------------------------------------------------
export const T = { T: 0 }

// reward pool  ------------------------------------------------------------------------------------
export class MockPool extends Pool {

  // in reward token
  interval    = FUND_INTERVAL
  portion     = 2500
  remaining   = 120
  balance     = this.portion

  // in lp token
  last_update = 0
  lifetime    = 0
  locked      = 0

  log: Log
  constructor (log: Log) {
    this.log = log
  }

  update () {
    this.log.now.textContent = `block ${T.T}`
    this.log.balance.textContent = `reward budget: ${this.balance.toFixed(3)}`
    this.log.remaining.textContent = `${this.remaining} days remaining`

    this.lifetime += this.locked
    if (T.T % this.interval == 0) {
      console.info('fund', this.portion, this.remaining)
      if (this.remaining > 0) {
        this.balance += this.portion
        this.remaining -= 1
      }
    }
  }
}

export class MockUser extends User {
  ui: UIContext

  pool:    Pool
  name:    string
  balance: number

  last_update = 0
  lifetime    = 0
  locked      = 0
  age         = 0
  cooldown    = THRESHOLD
  earned      = 0
  claimed     = 0
  claimable   = 0
  
  waited = 0
  last_claimed = 0

  constructor (ui: UIContext, pool: Pool, name: string, balance: number) {
    super(ui, pool, name, balance)
    this.ui      = ui
    this.pool    = pool
    this.name    = name
    this.balance = balance
  }

  colors () {
    return COLORS(this.pool, this)
  }

  lock (amount: number) {
    this.last_update = T.T
    this.locked += amount
    this.pool.locked += amount

    this.ui.log.add('locks', this.name, amount)
    this.ui.current.add(this)
    this.ui.stacked.add(this)
  }

  retrieve (amount: number) {
    if (this.locked < amount) return

    this.last_update = T.T
    this.locked -= amount

    this.ui.log.add('retrieves', this.name, amount)

    if (this.locked === 0) this.ui.current.remove(this)
  }

  claim () {
    if (this.locked === 0) return

    if (this.cooldown > 0) return

    if (this.claimed > this.earned) {
      this.ui.log.add('crowded out A', this.name, undefined)
      return
    }

    const reward = this.earned - this.claimed

    if (reward > this.pool.balance) {
      this.ui.log.add('crowded out B', this.name, undefined)
      return
    }

    this.ui.log.add('claim', this.name, reward)
    this.claimed = this.earned
    this.pool.balance -= reward
    this.cooldown = COOLDOWN
    this.last_claimed = T.T
  }

  update () { // WARNING assumes elapsed=1 !

    this.lifetime += this.locked
    this.earned    = this.pool.balance * this.lifetime / this.pool.lifetime
    this.claimable = this.earned - this.claimed

    if (this.locked > 0) {
      this.age++
      if (this.cooldown > 0) {
        this.cooldown -= 1
      } else {
        if (this.claimable < 0) this.waited++;
        if (this.last_claimed > USER_GIVES_UP_AFTER) {
          this.retrieve(this.locked)
        }
      }
    }

    this.ui.table.update(this)
  }
}

export type Users = Record<string, User>

// entry point -------------------------------------------------------------------------------------
export default function initMock (ui: UIContext) {
  const pool = new MockPool(ui.log)
  const users: Users = {}
  for (let i = 0; i < MAX_USERS; i++) {
    const name    = `User${i}`
    const balance = Math.floor(Math.random()*MAX_INITIAL)
    users[name]   = new MockUser(ui, pool, name, balance)
  }
  return {pool, users}
}

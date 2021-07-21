import { COLORS } from './helpers'
import { Table, Log, PieChart, StackedPieChart } from './widgets'

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
export class Pool {

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

export interface UIContext {
  log:     Log
  table:   Table
  current: PieChart
  stacked: StackedPieChart
}

export class User {
  log:     Log
  table:   Table
  current: PieChart
  stacked: StackedPieChart

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
    this.pool = pool

    this.log     = ui.log
    this.table   = ui.table
    this.current = ui.current
    this.stacked = ui.stacked

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
    this.log.add('locks', this.name, amount)
    this.current.add(this)
    this.stacked.add(this)
  }

  retrieve (amount: number) {
    if (this.locked < amount) return

    this.last_update = T.T
    this.locked -= amount
    this.log.add('retrieves', this.name, amount)

    if (this.locked === 0) this.current.remove(this)
  }

  claim () {
    if (this.locked === 0) return

    if (this.cooldown > 0) return

    if (this.claimed > this.earned) {
      this.log.add('crowded out A', this.name, undefined)
      return
    }

    const reward = this.earned - this.claimed

    if (reward > this.pool.balance) {
      this.log.add('crowded out B', this.name, undefined)
      return
    }

    this.log.add('claim', this.name, reward)
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

    this.table.update(this)
  }
}

export type Users = Record<string, User>

// entry point -------------------------------------------------------------------------------------
export default function initMock (ui: UIContext) {
  const pool = new Pool(ui.log)
  const users: Users = {}
  for (let i = 0; i < MAX_USERS; i++) {
    const name    = `User${i}`
    const balance = Math.floor(Math.random()*MAX_INITIAL)
    users[name]   = new User(ui, pool, name, balance)
  }
  return {pool, users}
}

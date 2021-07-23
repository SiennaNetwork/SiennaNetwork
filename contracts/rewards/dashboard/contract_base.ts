import { UIContext } from './widgets'
import { COLORS } from './gruvbox'

// settings ----------------------------------------------------------------------------------------
export const TIME_SCALE          = 60
           , FUND_PORTIONS       = 120
           , FUND_PORTION        = 2500
           , FUND_INTERVAL       = 17280/TIME_SCALE
           , COOLDOWN            = FUND_INTERVAL
           , THRESHOLD           = FUND_INTERVAL
           , USER_GIVES_UP_AFTER = Infinity
           , MAX_USERS           = 20
           , MAX_INITIAL         = 1000

// root of time (warning, singleton!) --------------------------------------------------------------
export const T = { T: 0 }

class RPT {
  interval  = FUND_INTERVAL
  portion   = FUND_PORTION
  remaining = FUND_PORTIONS
  vest () {
    if (T.T % this.interval == 0) {
      console.info('fund', this.portion, this.remaining)
      if (this.remaining > 0) {
        this.portion
        this.remaining -= 1
        return this.portion
      }
    }
    return 0
  }
}

export class Pool {
  rpt = new RPT()

  ui:          UIContext
  last_update: number = 0
  lifetime:    number = 0
  locked:      number = 0
  balance:     number = this.rpt.vest()

  constructor (ui: UIContext) {
    this.ui = ui
  }
  update () {
    this.balance += this.rpt.vest()
    this.ui.log.now.textContent       = `block ${T.T}`
    this.ui.log.balance.textContent   = `reward budget: ${this.balance.toFixed(3)}`
    this.ui.log.remaining.textContent = `${this.rpt.remaining} days remaining`
  }
}

export class User {
  ui:           UIContext
  pool:         Pool
  name:         string
  balance:      number
  last_update:  number = 0
  lifetime:     number = 0
  locked:       number = 0
  age:          number = 0
  earned:       number = 0
  claimed:      number = 0
  claimable:    number = 0
  cooldown:     number = 0
  waited:       number = 0
  last_claimed: number = 0
  constructor (ui: UIContext, pool: Pool, name: string, balance: number) {
    this.ui      = ui
    this.pool    = pool
    this.name    = name
    this.balance = balance
  }
  update () {
    this.ui.table.update(this)
  }
  lock (amount: number) {
    this.ui.log.add('locks', this.name, amount)
    this.ui.current.add(this)
    this.ui.stacked.add(this)
  }
  retrieve (amount: number) {
    this.ui.log.add('retrieves', this.name, amount)
    if (this.locked === 0) this.ui.current.remove(this)
  }
  claim () {
    if (this.locked === 0) return 0

    if (this.cooldown > 0) return 0

    if (this.claimed > this.earned) {
      this.ui.log.add('crowded out A', this.name, undefined)
      return 0
    }

    const reward = this.earned - this.claimed
    if (reward > this.pool.balance) {
      this.ui.log.add('crowded out B', this.name, undefined)
      return 0
    }

    this.ui.log.add('claim', this.name, reward)

    return reward
  }

  colors () {
    return COLORS(this.pool, this)
  }
}

export type Users = Record<string, User>

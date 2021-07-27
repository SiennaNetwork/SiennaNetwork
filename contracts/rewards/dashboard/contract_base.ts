import { UIContext } from './widgets'
import { COLORS } from './gruvbox'

// settings ----------------------------------------------------------------------------------------
export const TIME_SCALE          = 60
           , FUND_PORTIONS       = 140
           , DIGITS              = 1000000
           , DIGITS_INV          = Math.log10(DIGITS)
           , FUND_PORTION        = 2500 * DIGITS
           , FUND_INTERVAL       = 17280/TIME_SCALE
           , COOLDOWN            = FUND_INTERVAL/24
           , THRESHOLD           = FUND_INTERVAL
           , USER_GIVES_UP_AFTER = Infinity
           , MAX_USERS           = 20
           , MAX_INITIAL         = 10000

export const format = {
  integer:    (x:number) => String(x),
  decimal:    (x:number) => (x/DIGITS).toFixed(DIGITS_INV),
  percentage: (x:number) => `${format.decimal(x)}%`
}

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
  claimed:     number = 0
  cooldown:    number = 0
  threshold:   number = 0

  constructor (ui: UIContext) {
    this.ui = ui
  }
  update () {
    this.balance += this.rpt.vest()
    this.ui.log.now.setValue(T.T)

    this.ui.log.lifetime.setValue(this.lifetime)
    this.ui.log.locked.setValue(this.locked)

    this.ui.log.balance.setValue(format.decimal(this.balance))
    this.ui.log.claimed.setValue(format.decimal(this.claimed))
    this.ui.log.remaining.setValue(this.rpt.remaining)

    this.ui.log.cooldown.setValue(this.cooldown)
    this.ui.log.threshold.setValue(this.threshold)
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
  share:        number = 0
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
    throw new Error('not implemented')
  }
  doClaim (reward: number) { // stupid typescript inheritance constraints
    console.debug(this.name, 'claim', reward)
    if (reward <= 0) return 0

    if (this.locked === 0) return 0

    if (this.cooldown > 0 || this.age < THRESHOLD) return 0

    if (this.claimed > this.earned) {
      this.ui.log.add('crowded out A', this.name, undefined)
      return 0
    }

    if (reward > this.pool.balance) {
      this.ui.log.add('crowded out B', this.name, undefined)
      return 0
    }

    this.pool.balance -= reward
    this.ui.log.add('claim', this.name, reward)
    console.debug('claimed:', reward)
    return reward
  }

  colors () {
    return COLORS(this.pool, this)
  }
}

export type Users = Record<string, User>

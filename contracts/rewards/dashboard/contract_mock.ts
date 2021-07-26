import { UIContext } from './widgets'
import { T, Pool, User, Users } from './contract_base'

// settings ----------------------------------------------------------------------------------------
const TIME_SCALE          = 30
const COOLDOWN            = 17280/TIME_SCALE
const FUND_INTERVAL       = 17280/TIME_SCALE
const THRESHOLD           = 17280/TIME_SCALE
const USER_GIVES_UP_AFTER = Infinity
const MAX_USERS           = 100
const MAX_INITIAL         = 1000

// reward pool  ------------------------------------------------------------------------------------
export class MockPool extends Pool {
  update () {
    super.update()
    this.lifetime += this.locked
  }
}

export class MockUser extends User {
  cooldown    = THRESHOLD

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
    super.update()
  }

  lock (amount: number) {
    this.last_update = T.T
    this.locked += amount
    this.pool.locked += amount
    super.lock(amount)
  }

  retrieve (amount: number) {
    if (this.locked < amount) return
    super.retrieve(amount)
    this.last_update = T.T
    this.locked -= amount
  }

  claim () {
    const reward = super.claim()
    this.claimed = this.earned
    this.pool.balance -= reward
    this.cooldown = COOLDOWN
    this.last_claimed = T.T
    return reward
  }
}

// entry point -------------------------------------------------------------------------------------
export default function initMock (ui: UIContext) {
  const pool = new MockPool(ui)
  const users: Users = {}
  for (let i = 0; i < MAX_USERS; i++) {
    const name    = `User${i}`
    const balance = Math.floor(Math.random()*MAX_INITIAL)
    users[name]   = new MockUser(ui, pool, name, balance)
  }
  return {pool, users}
}

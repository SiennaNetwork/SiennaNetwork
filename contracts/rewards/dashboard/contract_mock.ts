import { UIContext } from './widgets'
import {
  T, Pool, User, Users,
  COOLDOWN, THRESHOLD, USER_GIVES_UP_AFTER, MAX_USERS, MAX_INITIAL
} from './contract_base'

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
    const reward = this.doClaim(this.earned - this.claimed)
    if (reward > 0) {
      this.claimed += reward
      this.cooldown = COOLDOWN
      this.last_claimed = T.T
    }
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

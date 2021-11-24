import { h, append } from '../helpers'
import Component from '../Component'
import ContractComponent from './Contract'
import Field  from '../widgets/Field'
import Button from '../widgets/Button'
import Pie    from '../widgets/PieChart'

export class Rewards extends ContractComponent {
  #dashboard: any = null
  get dashboard () { return this.#dashboard }
  set dashboard (v: any) { this.#dashboard = v }

  ui = {
    title: this.add(h('header', { textContent: 'Rewards' })),
    row:   this.add(h('div', { className: 'Row', style: 'flex-grow:0;flex-shrink:0' }))
  }

  totals = append(this.ui.row)(h('div'))

  stakedPie = append(this.totals)(Pie())
  volumePie = append(this.totals)(Pie())

  closed = append(this.totals)(Field('Closed', 'no'))
  staked = append(this.totals)(Field('Staked', 0))
  volume = append(this.totals)(Field('Volume', 0))

  updated = append(this.totals)(Field('Updated', 0))
  bonding = append(this.totals)(Field('Bonding', 0))

  unlocked =    append(this.totals)(Field('Unlocked',    0))
  distributed = append(this.totals)(Field('Distributed', 0))
  budget =      append(this.totals)(Field('Budget',      0))

  users = append(this.ui.row)(users(this))

  initMsg = {
    config: {
      reward_token: { address: "SIENNA_addr",  code_hash: "SIENNA_hash"  },
      lp_token:     { address: "LPTOKEN_addr", code_hash: "LPTOKEN_hash" }
    }
  }

  #id: string = ""
  get id () { return this.#id }
  set id (id: string) {
    this.#id = id
    this.ui.title.textContent = `Rewards ${id}`
  }

  deposit (id: string, amount: BigInt) {
    console.debug('deposit', id, amount)

    this.dashboard.lpToken.handle(id, {
      increase_allowance:{spender:this.addr,amount:amount.toString()}
    })

    this.handle(id, {
      rewards:{deposit:{amount:amount.toString()}}
    })

    this.update()
    this.dashboard.lpToken.update()
  }

  withdraw (id: string, amount: BigInt) {
    this.handle(id, {
      rewards:{withdraw:{amount:amount.toString()}}
    })

    this.update()
    this.dashboard.lpToken.update()
  }

  claim (id: string) {
    this.handle(id, {
      rewards:{claim:{}}
    })

    this.update()
    this.dashboard.sienna.update()
  }

  update () {}
}

export class Users extends Component {
  #pool: Rewards|null = null
  get pool () { return this.#pool as Rewards }
  set pool (v: Rewards) { this.#pool = v }

  ui = {
    addUser: this.add(addUser(this)),
  }

  register (id: string) {
    const u = user(this, id)
    console.log(u)
    this.add(u)
    this.pool.handle(id, {set_viewing_key:{key:""}})
    u.update()
  }
}

export class AddUser extends Component {
  #users: Users|null = null
  get users () { return this.#users as Users }
  set users (v: Users) { this.#users = v }

  ui = {
    id:         this.add(Field('New user', '')),
    deposit1:   this.add(Button('+1',   () => this.addUser(1n))),
    deposit100: this.add(Button('+100', () => this.addUser(100n))),
  }

  addUser (stake: BigInt) {
    console.debug('addUser', stake)
    const id = this.users.pool.dashboard.addUser(stake)
    this.users.pool.deposit(id, stake)
  }

}

export class User extends Component {
  #users: Users|null = null
  get users () { return this.#users as Users }
  set users (v: Users) { this.#users = v }

  #id: string = ""
  get id () { return this.#id }
  set id (id: string) {
    this.#id = id
    this.ui.id.value = `${id}`
  }

  ui = { 
    id:                       this.add(Field('ID',                   this.id)),
    staked:                   this.add(Field('Staked',                     0)),
    volume:                   this.add(Field('Volume',                     0)),
    starting_pool_volume:     this.add(Field('Pool volume at entry',       0)),
    accumulated_pool_volume:  this.add(Field('Pool volume since entry',    0)),
    starting_pool_rewards:    this.add(Field('Reward budget at entry',     0)),
    accumulated_pool_rewards: this.add(Field('Rewards vested since entry', 0)),
    bonding:                  this.add(Field('Remaining bonding period',   0)),
    earned:                   this.add(Field('Earned rewards',             0)),

    withdraw100: this.add(Button( '-100', () => this.withdraw(100n))),
    withdraw1:   this.add(Button(   '-1', () => this.withdraw(1n))),
    deposit1:    this.add(Button(   '+1', () => this.deposit(1n))),
    deposit100:  this.add(Button( '+100', () => this.deposit(100n))),
    claim:       this.add(Button('Claim', () => this.claim())),
  }

  deposit (amount: BigInt) {
    this.users.pool.dashboard.lpToken.mint(this.id, amount)
    this.users.pool.deposit(this.id, amount)
    this.update()
  }

  withdraw (amount: BigInt) {
    this.users.pool.withdraw(this.id, amount)
    this.update()
  }

  claim () {
    this.users.pool.claim(this.id)
    this.update()
  }

  update () {
    const {rewards:{user_info}} = this.users.pool.query({
      rewards:{user_info:{at:0,address:this.id,key:""}}
    })

    this.ui.staked.value                   = user_info.staked
    this.ui.volume.value                   = user_info.volume
    this.ui.starting_pool_volume.value     = user_info.starting_pool_volume
    this.ui.accumulated_pool_volume.value  = user_info.accumulated_pool_volume
    this.ui.starting_pool_rewards.value    = user_info.starting_pool_rewards
    this.ui.accumulated_pool_rewards.value = user_info.accumulated_pool_rewards
    this.ui.bonding.value                  = user_info.bonding
    this.ui.earned.value                   = user_info.earned
  }

}

customElements.define('x-rewards', Rewards)

export default function rewards (dashboard: any, id: string) {
  return h('x-rewards', { id, className: `Outside Module Rewards ${id}`, dashboard })
}

customElements.define('x-users', Users)

export function users (pool: Rewards) {
  return h('x-users', { pool, className: 'Outside Users' })
}

customElements.define('x-add-user', AddUser)

export function addUser (users: Users) {
  return h('x-add-user', { users, className: 'Outside AddUser' })
}

customElements.define('x-user', User)

export function user (users: Users, id: string): User {
  return h('x-user', { users, id, className: 'Outside User' }) as User
}

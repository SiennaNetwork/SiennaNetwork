import { h, append, format } from '../helpers'
import Component from '../Component'
import ContractComponent from '../Cosmos'
import Field  from '../widgets/Field'
import Button from '../widgets/Button'
import Pie    from '../widgets/PieChart'

export default class Rewards extends ContractComponent {

  static TAG   = 'x-rewards'
  static _     = customElements.define(this.TAG, this)
  static CLASS = 'Outside Module Rewards'
  static make  = (dashboard: any, id: string) => h(this.TAG, { className: this.CLASS, dashboard, id })

  #dashboard: any = null
  get dashboard () { return this.#dashboard }
  set dashboard (v: any) { this.#dashboard = v }

  ui = {
    title: this.add(h('header', { textContent: 'Rewards' })),
    row:   this.add(h('div', { className: 'Row', style: 'flex-grow:0;flex-shrink:0' }))
  }

  totals = append(this.ui.row)(h('div', {style:'display:flex;flex-flow:column nowrap'}))

  stakedPie = append(this.totals)(Pie.make(this, 'staked'))
  volumePie = append(this.totals)(Pie.make(this, 'volume'))

  resize = () => {
    this.stakedPie.resize()
    this.volumePie.resize()
  }

  staked = append(this.totals)(Field('Staked', 0))
  volume = append(this.totals)(Field('Volume', 0))

  updated = append(this.totals)(Field('Updated', 0))
  bonding = append(this.totals)(Field('Bonding', 0))

  unlocked    = append(this.totals)(Field('Unlocked',    0))
  distributed = append(this.totals)(Field('Distributed', 0))
  budget      = append(this.totals)(Field('Budget',      0))

  closed = append(this.totals)(Field('Closed', 'no'))
  close = append(this.totals)(Button('Close', () => {}))
  drain = append(this.totals)(Button('Drain', () => {}))

  userList     = append(this.ui.row)(h('div', { className: 'Outside Inside Users' }))
  newUser      = append(this.userList)(h('div', { className: 'Outside Inside AddUser' }))
  newUserLabel = append(this.newUser)(Field('New user', ''))
  deposit1     = append(this.newUser)(Button('+1',   () => this.addUser(1n)))
  deposit100   = append(this.newUser)(Button('+100', () => this.addUser(100n)))

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

  users: Record<string, User> = {}

  register (id: string) {
    append(this.userList)(this.users[id] = User.make(this, id))
    this.handle(id, {set_viewing_key:{key:""}})
  }

  addUser (stake: BigInt) {
    console.debug('addUser', stake)
    const id = this.dashboard.addUser(stake)
    this.deposit(id, stake)
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
    this.users[id].update()

    this.stakedPie.add
    this.volumePie.add
  }

  withdraw (id: string, amount: BigInt) {
    this.handle(id, {
      rewards:{withdraw:{amount:amount.toString()}}
    })

    this.update()
    this.dashboard.lpToken.update()
    this.users[id].update()
  }

  claim (id: string) {
    this.handle(id, {
      rewards:{claim:{}}
    })

    this.update()
    this.dashboard.sienna.update()
    this.users[id].update(this.dashboard.environment.time)
  }

  nextEpoch (next_epoch: number) {
    this.handle("Admin", {rewards:{begin_epoch:{next_epoch}}})
    this.update()
    this.dashboard.sienna.update()
  }

  update () {
    const at = this.dashboard.environment.time
    const {rewards:{pool_info}} = this.query({rewards:{pool_info:{at}}})
    console.log(pool_info)
    this.closed.value = pool_info.closed||'no'
    this.staked.value = pool_info.staked
    this.volume.value = pool_info.volume
    this.updated.value = pool_info.updated
    this.bonding.value = pool_info.bonding
    this.unlocked.value    = format.SIENNA(BigInt(pool_info.unlocked))
    this.distributed.value = format.SIENNA(BigInt(pool_info.distributed))
    this.budget.value      = format.SIENNA(BigInt(pool_info.budget))
    for (const user of Object.values(this.users)) user.update(at)
    this.stakedPie.render()
    this.volumePie.render()
  }
}

export class User extends Component {

  static TAG   = 'x-user'
  static _     = customElements.define(this.TAG, this)
  static CLASS = 'Outside User'
  static make  = (pool: Rewards, id: string) => h('x-user', { pool, id, className: 'Outside User' }) as User

  #pool: Rewards|null = null
  get pool () { return this.#pool as Rewards }
  set pool (v: Rewards) { this.#pool = v }

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
    this.pool.dashboard.lpToken.mint(this.id, amount)
    this.pool.deposit(this.id, amount)
    this.update()
  }

  withdraw (amount: BigInt) {
    this.pool.withdraw(this.id, amount)
    this.update()
  }

  claim () {
    this.pool.claim(this.id)
    this.update()
  }

  update (at = this.pool.dashboard.environment.time) {
    const {rewards:{user_info}} = this.pool.query({
      rewards:{user_info:{at,address:this.id,key:""}}
    })

    this.ui.staked.value                   = user_info.staked
    this.ui.volume.value                   = user_info.volume
    this.ui.starting_pool_volume.value     = user_info.starting_pool_volume
    this.ui.accumulated_pool_volume.value  = user_info.accumulated_pool_volume
    this.ui.starting_pool_rewards.value    = format.SIENNA(user_info.starting_pool_rewards)
    this.ui.accumulated_pool_rewards.value = format.SIENNA(user_info.accumulated_pool_rewards)
    this.ui.bonding.value                  = user_info.bonding
    this.ui.earned.value                   = format.SIENNA(user_info.earned)
  }

}

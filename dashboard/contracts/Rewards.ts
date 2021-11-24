import { h } from '../helpers'
import Component from '../Component'
import ContractComponent from './Contract'
import Field  from '../widgets/Field'
import Button from '../widgets/Button'
import Pie    from '../widgets/PieChart'

export class Rewards extends ContractComponent {
  #dashboard: any = null
  get dashboard () { return this.#dashboard }
  set dashboard (v: any) { this.#dashboard = v }

  closed: [number, string] | null = null
  staked:      number = 0
  volume:      number = 0
  updated:     number = 0
  bonding:     number = 0
  unlocked:    number = 0
  distributed: number = 0
  budget:      number = 0

  ui = {
    title:     this.add(h('header', { textContent: 'Rewards' })),
    stakedPie: this.add(Pie()),
    volumePie: this.add(Pie()),

    closed: this.add(Field('Closed', this.closed||'no')),
    staked: this.add(Field('Staked', this.staked)),
    volume: this.add(Field('Volume', this.volume)),

    updated: this.add(Field('Updated', this.updated)),
    bonding: this.add(Field('Bonding', this.bonding)),

    unlocked:    this.add(Field('Unlocked',    this.unlocked)),
    distributed: this.add(Field('Distributed', this.distributed)),
    budget:      this.add(Field('Budget',      this.budget)),

    users: this.add(users(this))
  }

  initMsg = {
    config: {
      reward_token: { address: "SIENNA_addr", code_hash: "SIENNA_hash" }
    }
  }

  #id: string = ""
  get id () { return this.#id }
  set id (id: string) {
    this.#id = id
    this.ui.title.textContent = `Rewards ${id}`
  }

  update () {}

  deposit (id: string, amount: BigInt) {
    console.debug('deposit', id, amount)
    const msg = {rewards:{deposit:{amount:amount.toString()}}}
    const response = this.handle(id, msg)
    console.log(response)
    this.update()
  }
}

export class Users extends Component {
  #pool: Rewards|null = null
  get pool () { return this.#pool as Rewards }
  set pool (v: Rewards) { this.#pool = v }

  ui = {
    addUser: this.add(addUser(this)),
  }

  register (id: string) {
    this.add(user(this, id))
  }
}

export class AddUser extends Component {
  #users: Users|null = null
  get users () { return this.#users as Users }
  set users (v: Users) { this.#users = v }

  ui = {
    id:         this.add(Field('New user', '')),
    deposit1:   this.add(Button('+1',   () => this.addUser(1))),
    deposit100: this.add(Button('+100', () => this.addUser(100))),
  }

  addUser (stake: number) {
    this.users.pool.dashboard.addUser(this.users.pool, stake)
  }

}

export class User extends Component {

  #id: string = ""
  get id () { return this.#id }
  set id (id: string) {
    this.#id = id
    this.ui.id.textContent = `${id}`
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

    withdraw100: this.add(Button( '-100')),
    withdraw1:   this.add(Button(   '-1')),
    deposit1:    this.add(Button(   '+1')),
    deposit100:  this.add(Button( '+100')),
    claim:       this.add(Button('Claim')),
  }

  staked:                   number = 0
  pool_share:               number = 0
  volume:                   number = 0
  starting_pool_volume:     number = 0
  accumulated_pool_volume:  number = 0
  reward_share:             number = 0
  starting_pool_rewards:    number = 0
  accumulated_pool_rewards: number = 0
  earned:                   number = 0
  updated:                  number = 0
  elapsed:                  number = 0
  bonding:                  number = 0

  //constructor (parent: HTMLElement, public id: string) {
    //append(parent, this.root)

    //let x = this.add(h('div', { className: 'Row' }))
    //append(x, this.ui.staked.root)
    //append(x, this.ui.volume.root)
    ////append(this.ui.staked.value, this.ui.withdraw100.root)
    ////append(this.ui.staked.value, this.ui.withdraw1.root)
    ////append(this.ui.staked.value, this.ui.deposit1.root)
    ////append(this.ui.staked.value, this.ui.deposit100.root)

    //x = this.add(h('div', { className: 'Row' }))
    //append(x, this.ui.starting_pool_volume.root)
    //append(x, this.ui.accumulated_pool_volume.root)

    //x = this.add(h('div', { className: 'Row' }))
    //append(x, this.ui.starting_pool_rewards.root)
    //append(x, this.ui.accumulated_pool_rewards.root)

    //x = this.add(h('div', { className: 'Row' }))
    //append(x, this.ui.bonding.root)
    //append(x, this.ui.earned.root)

    //append(this.ui.earned.root, this.ui.claim.root)
  //}
}

customElements.define('x-rewards', Rewards)

export default function rewards (dashboard: any, id: string) {
  return h('x-rewards', { id, className: `Outside Rewards ${id}`, dashboard })
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

export function user (users: Users, id: string) {
  return h('x-user', { users, id, className: 'Outside User' })
}

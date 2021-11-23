import { h } from './helpers'
import Component, { ContractComponent } from './Component'
import Field  from './Field'
import Button from './Button'
import Pie    from './PieChart'

export class Rewards extends ContractComponent {

  ui = {
    title:
      this.add(h('header', { textContent: 'Rewards' })),
    stakedPie:
      this.add(Pie()),
    volumePie:
      this.add(Pie()),
    //addUser:
      //new AddUser(this.root, this)
  }

  initMsg = {
    config: {
      reward_token: { address: "", code_hash: "" }
    }
  }

  #id: string = ""
  get id () { return this.#id }
  set id (id: string) {
    this.#id = id
    this.ui.title.textContent = `Rewards ${id}`
  }

  closed: [number, string] | null = null
  staked:      number = 0
  volume:      number = 0
  updated:     number = 0
  bonding:     number = 0
  unlocked:    number = 0
  distributed: number = 0
  budget:      number = 0

  update () {}

  totals: Record<string, any> = {}
  users:  Record<string, User> = {}
  addUser (id: string) {
    //this.users[id] = new User(this.root, id)
  }
}

export class AddUser extends Component {

  ui = {
    id:         this.add(Field('New user', '')),
    deposit1:   this.add(Button('+1')),
    deposit100: this.add(Button('+100')),
  }

}

export class User extends Component {

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
export default function rewards (id: string) {
  return h('x-rewards', { id, className: `Outside Rewards ${id}` })
}

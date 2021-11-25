import { h, append, format } from '../helpers'
import Component             from '../Component'
import ContractComponent     from './Contract'
import Field                 from '../widgets/Field'
import schedule              from '../../settings/schedule.json'
import Button                from '../widgets/Button'

export class MGMT extends ContractComponent {

  static TAG   = 'x-mgmt'
  static CLASS = 'Outside Module MGMT'
  static make  = () =>
    h(this.TAG, { className: this.CLASS })

  initMsg = {
    schedule,
    token: ["SIENNA_addr", "SIENNA_hash"]
  }

  ui = {
    title: this.add(h('header', { textContent: 'MGMT' })),
    total: this.add(Field("Total", 0)),
    pools: this.add(h('div'))
  }

  total: BigInt = 0n

  update () {
    const {schedule:{schedule:{total, pools}}} = this.query({schedule:{}})
    this.total = total
    this.ui.total.value = format.SIENNA(total)
    this.ui.pools.innerHTML = ''
    for (const pool of pools) {
      append(this.ui.pools)(Field(`.${pool.name}`, format.SIENNA(pool.total)))
      if (pool.name === 'MintingPool') {
        for (const account of pool.accounts) {
          append(this.ui.pools)(Field(`..${account.name}`, format.SIENNA(account.amount)))
        }
      }
    }
  }
  
  launch () {
    this.handle("Admin",{"launch":{}})
    this.update()
  }

}

export class RPT extends ContractComponent {

  static TAG   = 'x-rpt'
  static CLASS = 'Outside Module RPT'
  static make  = (dashboard: any) =>
    h(this.TAG, { className: this.CLASS, dashboard })

  #dashboard: any = null
  get dashboard () { return this.#dashboard }
  set dashboard (v: any) { this.#dashboard = v }

  ui = {
    title:  this.add(h('header', { textContent: 'RPT' })),
    config: this.add(h('div'))
  }

  initMsg = {
    portion: "2500",
    config:  [["REWARDS_V3_addr","2500"]],
    token:   ["SIENNA_addr", "SIENNA_hash"],
    mgmt:    ["MGMT_addr", "MGMT_hash"]
  }

  update () {
    const {status} = this.query({status:{}})
    const {portion, config} = status
    this.ui.config.innerHTML = ''
    for (let [recipient, amount] of config) {
      recipient = recipient || '???'
      append(this.ui.config)(Field(recipient, amount))
    }
    this.dashboard.sienna.update()
  }

  vest () {
    console.log('vest')
    this.handle("", {vest: {}})
    this.update()
  }

  performMigration () {
    this.handle("Admin", {"configure":{"config":[["REWARDS_v4_addr","2500"]]}})
    this.update()
  }
}

export class Microservice extends Component {

  static TAG   = 'x-microservice'
  static CLASS = 'Outside Microservice'
  static make  = (dashboard: any) =>
    h(this.TAG, { className: this.CLASS, dashboard })

  #dashboard: any = null
  get dashboard () { return this.#dashboard }
  set dashboard (v: any) { this.#dashboard = v }

  epoch = 0

  ui = {
    title: this.add(h('header', { textContent: 'Microservice' })),
    epoch: this.add(Field('Epoch', this.epoch)),
    next:  this.add(Button('NEXT', () => this.nextEpoch()))
  }

  nextEpoch () {
    this.dashboard.rpt.vest()
    this.dashboard.rewards_v3.nextEpoch(this.epoch + 1)
    this.epoch += 1
    this.ui.epoch.value = this.epoch
  }

}

customElements.define('x-mgmt',         MGMT)
customElements.define('x-rpt',          RPT)
customElements.define('x-microservice', Microservice)

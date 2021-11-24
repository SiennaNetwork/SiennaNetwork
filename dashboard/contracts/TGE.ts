import { h, append, format } from '../helpers'
import Component             from '../Component'
import ContractComponent     from './Contract'
import Field                 from '../widgets/Field'
import schedule              from '../../settings/schedule.json'
import Button                from '../widgets/Button'

export class MGMT extends ContractComponent {

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
    for (const pool of pools) {
      this.add(Field(`.${pool.name}`, format.SIENNA(pool.total)))
      if (pool.name === 'MintingPool') {
        for (const account of pool.accounts) {
          this.add(Field(`..${account.name}`, format.SIENNA(account.amount)))
        }
      }
    }
  }

}

export class RPT extends ContractComponent {

  ui = {
    title:  this.add(h('header', { textContent: 'RPT' })),
    config: this.add(h('div'))
  }

  initMsg = {
    portion: "2500",
    config:  [["REWARDS_v3_addr","2500"]],
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
  }

  performMigration () {
    this.handle("Admin", {"configure":{"config":[["REWARDS_v4_addr","2500"]]}})
    this.update()
  }
}

customElements.define('x-mgmt', MGMT)
export function mgmt () {
  return h('x-mgmt', { className: 'Outside Module MGMT' })
}

customElements.define('x-rpt', RPT)
export function rpt () {
  return h('x-rpt', { className: 'Outside Module RPT' })
}

export class Microservice extends Component {

  epoch = 0

  ui = {
    title: this.add(h('header', { textContent: 'Microservice' })),
    epoch: this.add(Field('Epoch', this.epoch)),
    next:  this.add(Button('NEXT'))
  }

}

customElements.define('x-microservice', Microservice)
export function microservice () {
  return h('x-microservice', { className: 'Outside Microservice' })
}

import { h, append } from '../helpers'
import ContractComponent from './Contract'
import Field from '../widgets/Field'

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

customElements.define('x-rpt', RPT)
export default function rpt () {
  return h('x-rpt', { className: 'Outside RPT' })
}

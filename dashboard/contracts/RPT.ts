import { h } from '../helpers'
import ContractComponent from './Contract'
import Field from '../widgets/Field'

export class RPT extends ContractComponent {

  ui = {
    title: this.add(h('header', { textContent: 'RPT' }))
  }

  initMsg = {
    portion: "2500",
    config:  [["","2500"]],
    token:   ["SIENNA_addr", "SIENNA_hash"],
    mgmt:    ["MGMT_addr", "MGMT_hash"]
  }

  update () {
    const {status} = this.query({status:{}})
    const {portion, config} = status
    for (let [recipient, amount] of config) {
      recipient = recipient || '???'
      this.add(Field(recipient, amount))
    }
  }
}

customElements.define('x-rpt', RPT)
export default function rpt () {
  return h('x-rpt', { className: 'Outside RPT' })
}

import { h } from '../helpers'
import { ContractComponent } from '../Component'
import Field from '../widgets/Field'

export class RPT extends ContractComponent {

  ui = {
    title: this.add(h('header', { textContent: 'RPT' }))
  }

  initMsg = {
    portion: "2500",
    config:  [["","2500"]],
    token:   ["", ""],
    mgmt:    ["", ""]
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

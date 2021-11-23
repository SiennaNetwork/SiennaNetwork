import { h, encode, decode } from './helpers'
import Component from './Component'
import Field from './Field'
import schedule from '../settings/schedule.json'

export class MGMT extends Component {

  #contract: any
  setup (Contract: any) {
    this.#contract = new Contract()
    this.#contract.init(encode({
      schedule,
      token: ["", ""]
    }))
  }

  ui = {
    title: this.add(h('header', { textContent: 'MGMT' })),
    total: this.add(Field("Total", 0)),
    pools: this.add(h('div'))
  }

  update () {
    const {schedule:{schedule:{total, pools}}} =
      decode(this.#contract.query(encode({schedule:{}})))
    this.ui.total.value = total
    for (const pool of pools) {
      this.add(Field(`.${pool.name}`, pool.total))
      if (pool.name === 'MintingPool') {
        for (const account of pool.accounts) {
          this.add(Field(`..${account.name}`, account.amount))
        }
      }
    }
  }

}

customElements.define('x-mgmt', MGMT)
export default function mgmt () {
  return h('x-mgmt', { className: 'Outside MGMT' })
}

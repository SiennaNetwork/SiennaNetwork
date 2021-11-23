import { h, encode, decode } from './helpers'
import {ContractComponent} from './Component'
import Field from './Field'
import schedule from '../settings/schedule.json'

export class MGMT extends ContractComponent {

  initMsg = {
    schedule,
    token: ["", ""]
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

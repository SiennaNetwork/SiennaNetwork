import { h, format } from '../helpers'
import ContractComponent from './Contract'
import Field from '../widgets/Field'
import schedule from '../../settings/schedule.json'

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

customElements.define('x-mgmt', MGMT)
export default function mgmt () {
  return h('x-mgmt', { className: 'Outside MGMT' })
}

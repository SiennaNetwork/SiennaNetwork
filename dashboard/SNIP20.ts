import { h, encode } from './helpers'
import { ContractComponent } from './Component'
import field from './Field'

export class SNIP20 extends ContractComponent {

  ui = {
    title: this.add(h('header', { textContent: 'SNIP20' })),
    table: this.add(h('table'))
  }

  #id: string = ""
  get id () { return this.#id }
  set id (id: string) {
    this.#id = id
    this.initMsg.name = id
    this.initMsg.symbol = id
    this.ui.title.textContent = id
  }

  initMsg = {
    name:      this.id,
    symbol:    this.id,
    decimals:  6,
    prng_seed: ''
  }

  balances: Record<string, number> = {}
  displays: Record<string, any>  = {}
  addAccount (id: string, balance: number = 0) {
    this.balances[id] = balance
    this.displays[id] = this.add(field(id, `${balance} ${this.id}`))
  }

  update () {}

}

customElements.define('x-snip20', SNIP20)
export default function snip20 (id: string) {
  return h('x-snip20', { id, className: `Outside SNIP20 ${id}` })
}

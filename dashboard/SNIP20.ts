import { h, append, encode } from './helpers'
import field from './Field'

export class SNIP20 extends HTMLElement {

  root = this.attachShadow({ mode: 'open' })
  add  = append(this.root)

  ui = {
    title: this.add(h('header', { textContent: 'SNIP20' })),
    table: this.add(h('table'))
  }

  balances: Record<string, number> = {}
  displays: Record<string, any>  = {}
  addAccount (id: string, balance: number = 0) {
    this.balances[id] = balance
    this.displays[id] = this.add(field(id, `${balance} ${this.id}`))
  }

  #id: string = ""
  get id () { return this.#id }
  set id (id: string) { this.#id = id }

  #contract: any
  setup (Contract: any) {
    this.#contract = new Contract()
    this.#contract.init(encode({
      name: this.id,
      symbol: this.id,
      decimals: 6,
      prng_seed: ''
    }))
  }

}

customElements.define('x-snip20', SNIP20)
export default function snip20 (id: string) {
  return h('x-snip20', { id, className: `Module SNIP20 ${id}` })
}

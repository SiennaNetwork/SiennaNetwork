import { h, encode } from '../helpers'
import { ContractComponent } from '../Component'
import field from '../widgets/Field'

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
    prng_seed: '',
    config: { enable_mint: true }
  }

  users: Array<string> = []
  displays: Record<string, any> = {}
  register (id: string) {
    this.users.push(id)
    this.handle(id, {set_viewing_key:{key:""}})
    this.displays[id] = this.add(field(id, `0 ${this.id}`))
  }

  mint (id: string, amount: number) {
    this.users.push(id)
    this.handle("", {set_minters:{minters:[""]}})
    this.handle("", {mint:{recipient:id,amount:String(amount)}})
    this.update()
  }

  update () {
    for (const user of this.users) {
      const response = this.query({balance:{address:user,key:""}})
      if (response.viewing_key_error) {
        throw new Error(response.viewing_key_error)
      } else {
        this.displays[user].value = `${response.balance.amount} ${this.id}`
      }
    }
  }

}

customElements.define('x-snip20', SNIP20)
export default function snip20 (id: string) {
  return h('x-snip20', { id, className: `Outside SNIP20 ${id}` })
}

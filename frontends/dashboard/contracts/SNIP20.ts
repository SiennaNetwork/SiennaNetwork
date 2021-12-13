import { h } from '../helpers'
import field from '../widgets/Field'
import ContractComponent from '../Cosmos'

export default class SNIP20 extends ContractComponent {

  static TAG   = 'x-snip20'
  static _     = customElements.define(this.TAG, this)
  static CLASS = 'Outside Module SNIP20'
  static make  = (id: string) => h(this.TAG, { className: this.CLASS, id })

  #id: string = ""
  get id () { return this.#id }
  set id (id: string) {
    this.#id = id
    this.initMsg.name = id
    this.initMsg.symbol = id
    this.ui.title.textContent = id
  }

  ui = {
    title: this.add(h('header', { textContent: 'SNIP20' })),
    table: this.add(h('table'))
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

  mint (id: string, amount: BigInt) {
    console.trace('mint', id, amount)
    this.users.push(id)
    this.handle("Admin", {set_minters:{minters:["Admin"]}})
    this.handle("Admin", {mint:{recipient:id,amount:String(amount)}})
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

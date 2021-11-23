import { h, append, encode, decode } from './helpers'
import Component from './Component'
import Field from './Field'

export class RPT extends Component {

  ui = {
    title: this.add(h('header', { textContent: 'RPT' }))
  }

  portion = BigInt(2500)

  #contract: any
  setup (Contract: any) {
    this.#contract = new Contract()
    this.#contract.init(encode({
      portion: String(this.portion),
      config:  [["","2500"]],
      token:   ["", ""],
      mgmt:    ["", ""]
    }))
  }

  update () {
    const {status} = decode(this.#contract.query(encode({status:{}})))
    const {portion, config} = status
    this.portion = BigInt(portion)
    console.log(config)
    for (const [recipient, amount] of config) {
      this.add(Field(recipient, amount))
    }
  }
}

customElements.define('x-rpt', RPT)
export default function rpt () {
  return h('x-rpt', { className: 'Outside RPT' })
}

import { h, append } from './helpers'
import Field from './Field'

export class Microservice extends HTMLElement {
  root = this.attachShadow({ mode: 'open' })
  add = append(this.root)
  epoch = 0
  ui = {
    title: this.add(h('header', { textContent: 'Microservice' })),
    epoch: this.add(Field("Epoch", this.epoch))
  }
}

customElements.define('x-microservice', Microservice)
export default function microservice () {
  return h('x-microservice', { className: 'Module Microservice' })
}

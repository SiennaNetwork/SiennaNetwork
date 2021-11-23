import { h, append } from './helpers'

const PIE = 'x-pie'

export class PieChart extends HTMLElement {
  root = this.attachShadow({ mode: 'open' })
  add = append(this.root)
  ui = {
    canvas: this.add(h('canvas', { width: 1, height: 1 }))
  }
  //constructor (parent: HTMLElement) {
    //append(parent, this.root)
  //}
}

customElements.define(PIE, PieChart)

export default function pie () {
  return h(PIE)
}

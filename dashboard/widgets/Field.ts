import { h, append } from '../helpers'

/** A label-value pair */
export class Field extends HTMLElement {

  root = this.attachShadow({ mode: 'open' })
  add = append(this.root)
  ui = {
    label: this.add(h('label', {style:'font-weight:normal;flex-grow:1;white-space:nowrap'})),
    value: this.add(h('div',   {style:'font-weight:bold;white-space:nowrap'}))
  }

  #value: any = null

  get value () {
    return this.#value
  }

  set value (v: any) {
    this.#value = v
    this.ui.value.textContent = String(v)
  }

  #label: any = null

  get label () {
    return this.#label
  }

  set label (v: any) {
    this.#label = v
    this.ui.label.textContent = String(v)
  }

}

customElements.define('x-field', Field)

export default function field (label: string, value: any) {
  return h('x-field', {
    label,
    value,
    className: label.replace(/ /g, '_')
  })
}

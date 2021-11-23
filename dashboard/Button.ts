import { h, append } from './helpers'

export class Button extends HTMLElement {
}

customElements.define('x-button', Button)

export default function button (label: string) {
  return h('x-button', { label })
}

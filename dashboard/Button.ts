import { h } from './helpers'
import Component from './Component'

export class Button extends Component {
  set label (v: string) {
    console.log({v})
    this.base.innerText = v
  }
}

customElements.define('x-button', Button)

export default function button (label: string) {
  return h('x-button', { label })
}

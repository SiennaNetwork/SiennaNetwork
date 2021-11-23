import { h } from './helpers'
import Component from './Component'

export class Button extends Component {
  set label (v: string) {
    this.base.innerText = v
  }
}

customElements.define('x-button', Button)

export default function button (
  label: string,
  onclick = (event:any) => console.debug(event)
) {
  return h('x-button', { label, onclick })
}

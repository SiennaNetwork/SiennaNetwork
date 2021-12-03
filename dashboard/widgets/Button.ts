import { h } from '../helpers'
import Component from '../Component'

export default class Button extends Component {

  static TAG   = 'x-button'
  static _     = customElements.define('x-button', this)
  static CLASS = 'Outside Dashboard'
  static make  = (
    label: string,
    onclick = (event:any) => console.debug(event)
  ) => h('x-button', {
    label, onclick, className: label.replace(/ /g, '_')
  })

  set label (v: string) {
    this.base.innerText = v
  }

  update () {}
}

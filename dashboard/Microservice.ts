import { h } from './helpers'
import Component from './Component'
import Field     from './Field'
import Button    from './Button'

export class Microservice extends Component {

  epoch = 0

  ui = {
    title: this.add(h('header', { textContent: 'Microservice' })),
    epoch: this.add(Field('Epoch', this.epoch)),
    next:  this.add(Button('NEXT'))
  }

}

customElements.define('x-microservice', Microservice)
export default function microservice () {
  return h('x-microservice', { className: 'Outside Microservice' })
}

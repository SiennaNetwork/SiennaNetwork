import { h, append } from './helpers'
import Component from './Component'
import Field from './Field'

type Timer = ReturnType<typeof setTimeout>

export class Environment extends Component {

  time = 0
  rate = 1
  timer: Timer|null = null

  start () {
    this.timer = setInterval(this.update.bind(this), this.rate)
  }

  pause () {
    if (this.timer) clearInterval(this.timer)
    this.timer = null
  }

  update () {
    this.time += this.rate
  }

  ui = {
    title: this.add(h('header', { textContent: 'Environment' })),
    time:  this.add(Field('Time', this.time)),
    rate:  this.add(Field('Rate', this.rate)),
  }

}

customElements.define('x-environment', Environment)
export default function environment () {
  return h('x-environment', { className: 'Module' })
}

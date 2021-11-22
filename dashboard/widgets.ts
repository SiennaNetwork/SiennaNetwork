import { h, append } from './helpers'

export class Field {
  root  = h('div', { className: 'Field' })
  label = append(this.root, h('label'))
  value = append(this.root, h('div'))
  constructor (parent: HTMLElement, name: string, value?: any) {
    append(parent, this.root)
    this.label.textContent = name
    this.value.textContent = String(value)
  }
  setValue (value: any) {
    this.value.textContent = String(value)
  }
}

export class Button {
  root  = h('button', { className: 'Button' })
  constructor (parent: HTMLElement, name: string) {
    append(parent, this.root)
    this.root.textContent = name
  }
}

export class PieChart {
  root   = h('div', { className: 'pie' })
  canvas = append(this.root, h('canvas', { width: 1, height: 1 }))
  constructor (parent: HTMLElement) {
    append(parent, this.root)
  }
}

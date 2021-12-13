import { h, append } from './helpers'

export default class Component extends HTMLElement {

  root = this.attachShadow({ mode: 'open' })
  base = append(this.root)(h('div', { className: 'Inside' }))
  add  = append(this.base)

  // broken due to:
  // https://stackoverflow.com/questions/40181683/failed-to-execute-createelement-on-document-the-result-must-not-have-childr
  addTo (x: any) {
    x.appendChild(this)
  }

  constructor () {
    super()
    this.inheritStyles()
  }

  /** Ugly hack to inherit inlined style elements from document */
  private inheritStyles() {
    const styles = Array.from(document.head.querySelectorAll('style'))
    for (const style of styles) {
      const el = this.add(document.createElement('style'))
      el.innerHTML = style.innerHTML
    }
  }

  static get observedAttributes () {
    return [ 'class' ]
  }

  attributeChangedCallback (name: string, _oldValue: any, newValue: any) {
    switch (name) {
      case 'class':
        //this.classList.add('Outside')
        this.base.className = newValue.replace('Outside', 'Inside')
        //this.base.classList.add('Inside')
        break
    }
  }

}

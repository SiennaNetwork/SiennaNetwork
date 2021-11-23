import { append } from './helpers'
export default class Component extends HTMLElement {

  root = this.attachShadow({ mode: 'open' })

  add = append(this.root)

  // broken due to:
  // https://stackoverflow.com/questions/40181683/failed-to-execute-createelement-on-document-the-result-must-not-have-childr
  addTo (x: any) {
    x.appendChild(this)
  }

  constructor () {
    super()
    // ugly hack to inherit inlined style elements from document
    const styles = Array.from(document.head.querySelectorAll('style'))
    for (const style of styles) {
      const el = this.add(document.createElement('style'))
      el.innerHTML = style.innerHTML
    }
  }

}

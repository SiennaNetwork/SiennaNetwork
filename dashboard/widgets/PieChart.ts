import { h, append } from '../helpers'

export default class PieChart extends HTMLElement {

  static TAG   = 'x-pie'
  static _     = customElements.define(this.TAG, this)
  static CLASS = 'Outside Module Rewards'
  static make  = (pool: any, field: string) => h(this.TAG, {
    pool,
    field,
    style: 'align-self:center'
  })

  #pool: any = null
  get pool () { return this.#pool }
  set pool (v: any) { this.#pool = v }

  #field: any = null
  get field () { return this.#field }
  set field (v: any) { this.#field = v }

  root   = this.attachShadow({ mode: 'open' })

  add    = append(this.root)

  canvas = this.add(h('canvas', { width: 100, height: 100, style: 'border:1px solid black' }))

  resize () {
    this.canvas.width = this.canvas.height = 1
    const size = Math.max(
      Math.min(this.offsetWidth, this.offsetHeight),
      150
    )
    this.canvas.width = this.canvas.height = size
    this.render()
  }

  constructor () {
    super()
    this.render()
  }

  render () {
    requestAnimationFrame(()=>{
      // extract needed datum from user list
      // and sum the total
      let total: number = 0
      for (const user of Object.values(this.pool.users)) {
        const field = (user as any).ui[this.field]
        total += Number(field.value)
      }
      if (total === 0) return

      // prepare canvas
      const {width, height} = this.canvas
      const context = this.canvas.getContext('2d') as CanvasRenderingContext2D;

      // clear
      context.fillStyle = '#282828'
      context.fillRect(1, 1, width-2, height-2)

      // define center
      const centerX = width   / 2
      const centerY = height  / 2
      const radius  = centerX * 0.95

      // loop over segments
      let start = 0
      for (const id of Object.keys(this.pool.users).sort()) {
        const user = this.pool.users[id] as any
        const value = user.ui[this.field].value
        if (value === 0) continue
        const portion = value / total
        const end     = start + (2*portion)
        context.beginPath()
        context.moveTo(centerX, centerY)
        context.arc(centerX, centerY, radius, start * Math.PI, end * Math.PI)
        //context.moveTo(centerX, centerY)
        const [fillStyle, strokeStyle] = ['#98971a', '#000000']
        context.fillStyle = fillStyle
        context.strokeStyle = strokeStyle//'#000'//'rgba(255,255,255,0.5)'
        //context.strokeStyle = fillStyle//strokeStyle
        context.lineWidth = 0.8
        context.fill()
        context.stroke()
        start = end
      }
    })
  }
}

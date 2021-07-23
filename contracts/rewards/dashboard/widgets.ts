import { h, addTo } from './helpers'
import { T, User, Users, DIGITS, DIGITS_INV } from './contract_base'

// killswitches for gui components -----------------------------------------------------------------
export const NO_HISTORY = true
export const NO_TABLE   = false

// handles to dashboard components that can be passed into User/Pool objects -----------------------
// normally we'd do this with events but this way is simpler
export interface UIContext {
  log:     Log
  table:   Table
  current: PieChart
  stacked: StackedPieChart
}

// log of all modeled events -----------------------------------------------------------------------
export class Log {
  root      = h('div', { className: 'history' })
  now       = addTo(this.root, h('h1'))
  balance   = addTo(this.root, h('h1'))
  remaining = addTo(this.root, h('h2'))
  body      = addTo(this.root, h('ol'))
  add (event: string, name: string, amount: number|undefined) {
    if (NO_HISTORY) return
    if (amount) {
      this.body.insertBefore(
        h('div', { innerHTML: `<b>${name}</b> ${event} ${amount}LP` }),
        this.body.firstChild
      )
    } else {
      this.body.insertBefore(
        h('div', { innerHTML: `<b>${name}</b> ${event}` }),
        this.body.firstChild
      )
    }
  }
}

// table of current state --------------------------------------------------------------------------
interface Columns {
  name:         HTMLElement
  last_update:  HTMLElement
  lifetime:     HTMLElement
  share:        HTMLElement
  locked:       HTMLElement
  age:          HTMLElement
  earned:       HTMLElement
  claimed:      HTMLElement
  claimable:    HTMLElement
  cooldown:     HTMLElement
  last_claimed: HTMLElement
}

type Rows = Record<string, Columns>

export class Table {
  root: HTMLElement;
  rows: Rows = {};

  constructor () {
    this.root = document.createElement('table')
    if (NO_TABLE) return
  }

  init (users: Users) {
    addTo(this.root, h('thead', {},
      h('th', { textContent: 'name'         }),
      h('th', { textContent: 'last_update'  }),
      h('th', { textContent: 'age'          }),
      h('th', { textContent: 'locked'       }),
      h('th', { textContent: 'lifetime'     }),
      h('th', { textContent: 'share'        }),
      h('th', { textContent: 'earned'       }),
      h('th', { textContent: 'claimed'      }),
      h('th', { textContent: 'claimable'    }),
      h('th', { textContent: 'cooldown'     }),
      h('th', { textContent: 'last_claimed' })
    ))
    for (const name of Object.keys(users)) {
      this.addRow(name)
    }
  }

  addRow (name: string) {
    if (NO_TABLE) return
    const row = addTo(this.root, h('tr'))
    const rows = this.rows[name] = {
      name:         addTo(row, h('td', { style: 'font-weight:bold', textContent: name })),
      last_update:  addTo(row, h('td')),
      age:          addTo(row, h('td')),
      locked:       addTo(row, h('td')),
      lifetime:     addTo(row, h('td')),
      share:        addTo(row, h('td')),
      earned:       addTo(row, h('td')),
      claimed:      addTo(row, h('td')),
      claimable:    addTo(row, h('td')),
      cooldown:     addTo(row, h('td')),
      last_claimed: addTo(row, h('td'))
    }
    rows.claimable.style.fontWeight = 'bold'
    addTo(this.root, row)
    return rows
  }

  update (user: User) {
    if (NO_TABLE) return
    this.rows[user.name].last_update.textContent =
      String(user.last_update)
    this.rows[user.name].locked.textContent =
      String(user.locked)
    this.rows[user.name].lifetime.textContent =
      String(user.lifetime)
    this.rows[user.name].share.textContent =
      (100 * user.lifetime / user.pool.lifetime).toFixed(3) + '%'
    this.rows[user.name].age.textContent =
      String(user.age)
    this.rows[user.name].earned.textContent =
      (user.earned/DIGITS).toFixed(DIGITS_INV)
    this.rows[user.name].claimed.textContent =
      (user.claimed/DIGITS).toFixed(DIGITS_INV)
    this.rows[user.name].claimable.textContent =
      (user.claimable/DIGITS).toFixed(DIGITS_INV)
    const [fill, stroke] = user.colors()
    this.rows[user.name].claimable.style.backgroundColor =
      fill
    this.rows[user.name].claimable.style.color =
      stroke
    //table.rows[user.name].waited.textContent = user.age === 0 ? '' : (100*user.waited/user.age).toFixed(1) + '%'
    this.rows[user.name].cooldown.textContent =
      String(user.cooldown)
    this.rows[user.name].last_claimed.textContent =
      (user.cooldown === 0 && user.claimable < 0)
        ? String(user.last_claimed - T.T)
        : ''
  }
}

type Values = Record<string, number>
export class PieChart {
  root:   HTMLElement;
  canvas: HTMLCanvasElement;

  users: Users = {};
  total: number = 0;
  field: string;

  constructor (_name: string, field: string) {
    this.field  = field
    this.root   = h('div', { className: `pie ${field}` })
    this.canvas = addTo(this.root, h('canvas', { width: 1, height: 1 })) as HTMLCanvasElement
  }

  add (user: User) {
    this.users[user.name] = user
  }

  remove (user: User) {
    delete this.users[user.name]
  }

  resize () {
    this.canvas.width = this.canvas.height = 1
    const size = Math.min(this.root.offsetWidth, this.root.offsetHeight)
    this.canvas.width = this.canvas.height = size
    this.render()
  }

  render () {
    requestAnimationFrame(()=>{
      // extract needed datum from user list
      // and sum the total
      const values: Values = {}
      let total: number = 0
      for (const user of Object.values(this.users)) {
        const value = (user as any)[this.field]
        if (value) {
          total += value
          values[user.name] = value } }
      if (total === 0) return

      // prepare canvas
      const {width, height} = this.canvas
      const context = this.canvas.getContext('2d') as CanvasRenderingContext2D;

      // clear
      context.fillStyle = '#282828'
      context.fillRect(1, 1, width-2, height-2)

      // define center
      const centerX = width  / 2
      const centerY = height / 2
      const radius  = centerX * 0.95

      // loop over segments
      let start = 0
      for (const name of Object.keys(this.users).sort()) {
        const value = values[name]
        if (value) {
          const portion = value / total
          const end     = start + (2*portion)
          context.beginPath()
          context.moveTo(centerX, centerY)
          context.arc(centerX, centerY, radius, start * Math.PI, end * Math.PI)
          //context.moveTo(centerX, centerY)
          const [fillStyle, strokeStyle] = this.users[name].colors()
          context.fillStyle = fillStyle
          context.lineWidth = 0.8
          context.strokeStyle = strokeStyle// '#000'//rgba(255,255,255,0.5)'
          context.fill()
          context.stroke()
          start = end } } }) } }

export class StackedPieChart {
  root:   HTMLElement;
  canvas: HTMLCanvasElement;

  users: Users = {};
  add (user: User) {
    this.users[user.name] = user }
  remove (user: User) {
    delete this.users[user.name] }

  constructor () {
    this.root   = h('div', { className: `pie stacked` })
    this.canvas = addTo(this.root, h('canvas', { width: 1, height: 1 })) as HTMLCanvasElement }

  resize () {
    this.canvas.width = this.canvas.height = 1
    const size = Math.min(this.root.offsetWidth, this.root.offsetHeight)
    this.canvas.width = this.canvas.height = size
    this.render() }

  render () {
    requestAnimationFrame(()=>{
      // extract needed datum from user list
      // and sum the total
      let total: number = 0
      for (const user of Object.values(this.users)) {
        total += user.lifetime
      }
      if (total === 0) return

      // prepare canvas
      const {width, height} = this.canvas
      const context = this.canvas.getContext('2d') as CanvasRenderingContext2D;

      // clear
      context.fillStyle = '#282828'
      context.fillRect(1, 1, width-2, height-2)

      // define center
      const centerX = width  / 2
      const centerY = height / 2
      const radius  = centerX * 0.95

      // loop over segments
      let start = 0
      for (const name of Object.keys(this.users).sort()) {
        const user = this.users[name]
        if (user.lifetime === 0) continue
        const portion = user.lifetime / total
        const end     = start + (2*portion)
        context.beginPath()
        context.moveTo(centerX, centerY)
        context.arc(centerX, centerY, radius, start * Math.PI, end * Math.PI)
        //context.moveTo(centerX, centerY)
        const [fillStyle, strokeStyle] = user.colors()
        context.fillStyle = fillStyle
        context.strokeStyle = strokeStyle//'#000'//'rgba(255,255,255,0.5)'
        //context.strokeStyle = fillStyle//strokeStyle
        context.lineWidth = 0.8
        context.fill()
        context.stroke()
        start = end } }) }

}

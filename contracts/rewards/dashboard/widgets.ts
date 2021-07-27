import { h, append, prepend } from './helpers'
import { T, User, Users, format } from './contract_base'

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

// Label + value
export class Field {
  root  = h('div', { className: 'field' })
  label = append(this.root, h('label'))
  value = append(this.root, h('div'))
  constructor (name: string, value?: any) {
    this.label.textContent = name
    this.value.textContent = String(value)
  }
  append (parent: HTMLElement) {
    parent.appendChild(this.root)
    return this
  }
  setValue (value: any) {
    this.value.textContent = String(value)
  }
}

// global values + log of all modeled events -------------------------------------------------------
export class Log {
  root      = h('div', { className: 'history' })
  body      = append(this.root, h('ol'))

  now       = new Field('block').append(this.root)

  locked    = new Field('liquidity now in pool').append(this.root)
  lifetime  = new Field('all liquidity ever in pool').append(this.root)

  balance   = new Field('available reward balance').append(this.root)
  claimed   = new Field('rewards claimed by users').append(this.root)
  remaining = new Field('remaining funding portions').append(this.root)

  threshold = new Field('initial age threshold').append(this.root)
  cooldown  = new Field('cooldown after claim').append(this.root)
  liquid    = new Field('pool liquidity ratio').append(this.root)

  add (event: string, name: string, amount: number|undefined) {
    if (NO_HISTORY) return
    if (amount) {
      prepend(this.body, h('div', { innerHTML: `<b>${name}</b> ${event} ${amount}LP` }))
    } else {
      prepend(this.body, h('div', { innerHTML: `<b>${name}</b> ${event}` }))
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
  lockedMinus:  HTMLElement
  lockedValue:  HTMLElement
  lockedPlus:   HTMLElement
  age:          HTMLElement
  earned:       HTMLElement
  claimed:      HTMLElement
  claimable:    HTMLElement
  cooldown:     HTMLElement
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
    append(this.root, h('thead', {},
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
    ))
    for (const name of Object.keys(users)) {
      this.addRow(name, users[name])
    }
  }

  addRow (name: string, user: User) {
    if (NO_TABLE) return
    const row = append(this.root, h('tr'))
    const locked      = h('td', { className: 'locked' })
        , lockedMinus = append(locked, h('button', {
                          textContent: '-',
                          onclick: () => user.retrieve(100)
                        }))
        , lockedValue = append(locked, h('span', {
                          textContent: ''
                        }))
        , lockedPlus  = append(locked, h('button', {
                          textContent: '+',
                          onclick: () => user.lock(100)
                        }))
    const rows = this.rows[name] = {
      name:         append(row, h('td', { style: 'font-weight:bold', textContent: name })),
      last_update:  append(row, h('td')),
      age:          append(row, h('td')),
      locked:       append(row, locked),
      lockedMinus, lockedValue, lockedPlus,
      lifetime:     append(row, h('td')),
      share:        append(row, h('td')),
      earned:       append(row, h('td')),
      claimed:      append(row, h('td')),
      claimable:    append(row, h('td', { className: 'claimable', onclick: () => {user.claim()} })),
      cooldown:     append(row, h('td')),
    }
    rows.claimable.style.fontWeight = 'bold'
    append(this.root, row)
    return rows
  }

  update (user: User) {
    if (NO_TABLE) return
    this.rows[user.name].last_update.textContent =
      format.integer(user.last_update)
    this.rows[user.name].lockedValue.textContent =
      format.integer(user.locked)
    this.rows[user.name].lifetime.textContent =
      format.integer(user.lifetime)
    this.rows[user.name].share.textContent =
      format.percentage(user.share)
    this.rows[user.name].age.textContent =
      format.integer(user.age)
    this.rows[user.name].earned.textContent =
      format.decimal(user.earned)
    this.rows[user.name].claimed.textContent =
      format.decimal(user.claimed)
    this.rows[user.name].claimable.textContent =
      format.decimal(user.claimable)
    this.rows[user.name].cooldown.textContent =
      format.integer(user.cooldown)

    const [fill, stroke] = user.colors()
    this.rows[user.name].earned.style.backgroundColor =
    this.rows[user.name].claimed.style.backgroundColor =
    this.rows[user.name].claimable.style.backgroundColor =
      fill
    this.rows[user.name].claimable.style.color =
      stroke
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
    this.canvas = append(this.root, h('canvas', { width: 1, height: 1 })) as HTMLCanvasElement
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
    this.canvas = append(this.root, h('canvas', { width: 1, height: 1 })) as HTMLCanvasElement }

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

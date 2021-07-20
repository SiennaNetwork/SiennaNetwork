import './style.css'
import Gruvbox from './gruvbox'

// root of time ------------------------------------------------------------------------------------
let T = 0

// optimization toggles ----------------------------------------------------------------------------
//
/* reset user age on claim, effectively implementing a cooldown
 * equal to the threshold period */
const RESET_AGE = true

/* additional cooldown after claiming*/
const COOLDOWN = false

const numberOfAccounts = 100
const initialUserBalance = 1000

// log of all modeled events -----------------------------------------------------------------------
export class History {
  root    = h('div', { className: 'history' })
  now       = addTo(this.root, h('h1'))
  balance   = addTo(this.root, h('h1'))
  remaining = addTo(this.root, h('h1'))
  body      = addTo(this.root, h('ol'))
  add (event: string, name: string, amount: number|undefined) {
    if (amount) {
      this.body.insertBefore(
        h('div', { innerHTML: `<b>${name}</b> ${event} ${amount}LP` }),
        this.body.firstChild) }
    else {
      this.body.insertBefore(
        h('div', { innerHTML: `<b>${name}</b> ${event}` }),
        this.body.firstChild) } } }
const log = new History()

// the rewards contract and its participants -------------------------------------------------------
class Pool {

  // in reward token
  interval    = 24
  portion     = 2500
  remaining   = 120
  balance     = this.portion

  // in lp token
  last_update = 0
  lifetime    = 0
  locked      = 0

  update () {
    log.now.textContent = `T=${T}`
    log.balance.textContent = `reward budget: ${this.balance}`
    this.lifetime += this.locked
    if (T % this.interval == 0) {
      console.log('fund', this.portion, this.remaining)
      if (this.remaining > 0) {
        this.balance += this.portion
        this.remaining -= 1
        log.remaining.textContent = `${this.remaining} days remaining`
      }
    }
  }}
const pool = new Pool()

class User {
  name: string;
  balance: number;

  last_update = 0
  lifetime    = 0
  locked      = 0
  age         = 0
  earned      = 0
  claimed     = 0
  claimable   = 0

  constructor (name: string, balance: number) {
    this.name = name
    this.balance = balance }
  lock (amount: number) {
    this.last_update = T
    this.locked += amount
    pool.locked += amount
    log.add('locks', this.name, amount)
    current.add(this)
    lifetime.add(this)
    earned.add(this) }
  unlock (amount: number) {
    if (this.locked < amount) return
    this.last_update = T
    this.locked -= amount
    log.add('unlocks', this.name, amount)
    if (this.locked === 0) current.remove(this) }
  claim () {
    if (this.locked === 0)
      return
    if (this.age < threshold)
      return
    if (this.claimed > this.earned) {
      log.add('crowded out A', this.name, undefined)
      return }
    const reward = this.earned - this.claimed
    if (reward > pool.balance) {
      log.add('crowded out B', this.name, undefined)
      return }
    log.add('claim', this.name, reward)
    this.claimed = this.earned
    pool.balance -= reward
    claimed.add(this)
    if (RESET_AGE) this.age = 0 }
  update () { // WARNING assumes elapsed=1 !
    this.lifetime += this.locked
    if (this.locked > 0) this.age++
    this.earned = pool.balance * this.lifetime / pool.lifetime
    this.claimable = this.earned - this.claimed
    table.rows[this.name].last_update.textContent = String(this.last_update)
    table.rows[this.name].locked.textContent      = String(this.locked)
    table.rows[this.name].lifetime.textContent    = String(this.lifetime)
    table.rows[this.name].age.textContent         = String(this.age)
    table.rows[this.name].earned.textContent      = String(this.earned)
    table.rows[this.name].claimed.textContent     = String(this.claimed)
    table.rows[this.name].claimable.textContent   = String(this.claimable)
    const [fill, stroke] = this.colors()
    table.rows[this.name].claimable.style.color = stroke
  }
  colors () {
    switch (true) {
      case this.claimable > pool.balance:
        return [Gruvbox.fadedRed,    Gruvbox.brightRed]
      case this.claimed > this.earned:
        return [Gruvbox.fadedOrange, Gruvbox.brightOrange]
      case this.age == threshold:
        return [Gruvbox.brightAqua,  Gruvbox.brightAqua]
      case this.age >  threshold:
        return [Gruvbox.fadedAqua,   Gruvbox.brightAqua]
      default:
        return [Gruvbox.dark0,       Gruvbox.light0] } } }

type Users  = Record<string, User>

const users: Users = {}

for (let i = 0; i < numberOfAccounts; i++) {
  const name    = `User${i}`
  const balance = Math.floor(Math.random()*initialUserBalance)
  users[name]   = new User(name, balance) }

// table of current state
interface Columns {
  name:        HTMLElement
  last_update: HTMLElement
  lifetime:    HTMLElement
  locked:      HTMLElement
  age:         HTMLElement
  earned:      HTMLElement
  claimed:     HTMLElement
  claimable:   HTMLElement }
type Rows = Record<string, Columns>
export class Table {
  root: HTMLElement;
  rows: Rows = {};
  constructor () {
    this.root = document.createElement('table')
    addTo(this.root, h('thead', {},
      h('th', { textContent: 'name'        }),
      h('th', { textContent: 'last_update' }),
      h('th', { textContent: 'lifetime'    }),
      h('th', { textContent: 'locked'      }),
      h('th', { textContent: 'age'         }),
      h('th', { textContent: 'earned'      }),
      h('th', { textContent: 'claimed'     }),
      h('th', { textContent: 'claimable'   })))
    for (const name of Object.keys(users)) {
      this.addRow(name) } }
  addRow (name: string) {
    const row = addTo(this.root, h('tr'))
    const rows = this.rows[name] = {
      name:        addTo(row, h('td', { style: 'font-weight:bold', textContent: name })),
      last_update: addTo(row, h('td')),
      lifetime:    addTo(row, h('td')),
      locked:      addTo(row, h('td')),
      age:         addTo(row, h('td')),
      earned:      addTo(row, h('td')),
      claimed:     addTo(row, h('td')),
      claimable:   addTo(row, h('td')) }
    rows.claimable.style.fontWeight = 'bold'
    addTo(this.root, row)
    return rows } }
const table = new Table()

// pie chart (TODO replace with streamgraph, difficulty might be multiple colors in same stream) ---
type Values = Record<string, number>
export class PieChart {
  root:   HTMLElement;
  title:  HTMLElement;
  label:  HTMLElement;
  canvas: HTMLCanvasElement;
  list:   HTMLElement;

  users: Users = {};
  total: number = 0;
  field: string;

  constructor (name: string, field: string) {
    this.field  = field
    this.root   = h('div', { className: `pie ${field}` })
    this.title  = addTo(this.root, h('h1', { textContent: name }))
    this.label  = addTo(this.root, h('h2'))
    this.canvas = addTo(this.root, h('canvas', { width: 1, height: 1 })) as HTMLCanvasElement
    this.list   = addTo(this.root, h('ol')) }

  add (user: User) {
    this.users[user.name] = user }
  remove (user: User) {
    delete this.users[user.name] }

  resize () {
    this.canvas.width = this.canvas.height = 1
    const size = Math.min(this.root.offsetWidth, this.root.offsetHeight)
    this.canvas.width = this.canvas.height = size
    this.render() }

  render () {
    // extract needed datum from user list
    // and sum the total
    const values: Values = {}
    let total: number = 0
    for (const user of Object.values(this.users)) {
      const value = (user as any)[this.field]
      if (value) {
        total += value
        values[user.name] = value } }
    this.label.textContent = String(total)
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
        const [fillStyle, strokeStyle] = users[name].colors()
        context.fillStyle = fillStyle
        context.strokeStyle = strokeStyle
        context.fill()
        context.stroke()
        start = end } } } }

const current   = new PieChart('Current',  'locked')
const lifetime  = new PieChart('Lifetime', 'lifetime')
const earned    = new PieChart('Earned',   'earned')
const claimed   = new PieChart('Claimed',  'claimed')

// pool globals
const threshold    = 240

;(function start () {

  // add components
  for (const el of [ current
                   , lifetime
                   , earned
                   , claimed
                   , log
                   , table /*,sparkline*/]) {
    addTo(document.body, el.root) }

  // one of these actions will be picked each turn
  const actions = {
    lock (user: User, amount: number) {
      user.lock(amount) },
    unlock (user: User, amount: number) {
      user.unlock(amount) },
    claim  (user: User) {
      user.claim() } }

  resize()
  window.addEventListener('resize', throttle(100, resize))

  update()

  function update () {
    // advance time
    T++

    // periodically fund pool and increment its lifetime
    pool.update()

    // increment lifetimes and ages
    for (const user of Object.values(users)) user.update()

    // perform random action from random account for random amount
    const action = Object.values(actions)[Math.floor(Math.random()*3)]
    const name   = Object.keys(users)[Math.floor(Math.random()*Object.keys(users).length)]
    const user   = users[name]
    const amount = Math.floor(Math.random()*user.balance)
    action(user, amount)

    // update charts
    for (const chart of [current
                        ,lifetime
                        ,earned
                        ,claimed]) { chart.render() }

    // rinse and repeat
    setTimeout(update, 100) }

  function resize () {
    current.resize()
    lifetime.resize()
    earned.resize()
    claimed.resize() } })()

// helpers
function throttle (t: number, fn: Function) {
  // todo replacing t with a function allows for implementing exponential backoff
  let timeout: any
  return function throttled (...args:any) {
    return new Promise(resolve=>{
      if (timeout) clearTimeout(timeout)
      timeout = after(t, ()=>resolve(fn(...args))) })}}
function after (t: number, fn: Function) {
  return setTimeout(fn, t)}
function h (element: string, attributes={}, ...content:any) {
  const el = Object.assign(document.createElement(element), attributes)
  for (const el2 of content) el.appendChild(el2)
  return el}
function addTo (parent: HTMLElement, child: HTMLElement) {
  return parent.appendChild(child) }

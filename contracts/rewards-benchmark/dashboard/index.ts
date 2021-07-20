import './style.css'
import Gruvbox from './gruvbox'
import * as RewardsBenchmark from './rewards-benchmark/sienna_rewards_benchmark_bg.wasm'
;((RewardsBenchmark as unknown as Function)()).then(console.info)
//console.log(RewardsBenchmark())
//console.log(new RewardsBenchmark())

// root of time ------------------------------------------------------------------------------------
let T = 0

// source of integer randomness --------------------------------------------------------------------
const random = (max: number) => Math.floor(Math.random()*max)
const pickRandom = (x: any) => x[random(x.length)]

// optimization toggles ----------------------------------------------------------------------------
const NO_TABLE        = true
const RESET_AGE       = true
const COOLDOWN        = 0
const FUND_INTERVAL   = 240
const THRESHOLD       = 240
const MAX_USERS       = 1000
const MAX_INITIAL     = 100
const UPDATE_INTERVAL = 1

// log of all modeled events -----------------------------------------------------------------------
class History {
  root      = h('div', { className: 'history' })
  now       = addTo(this.root, h('h1'))
  balance   = addTo(this.root, h('h1'))
  remaining = addTo(this.root, h('h2'))
  body      = addTo(this.root, h('ol'))
  add (event: string, name: string, amount: number|undefined) {} }
    //if (amount) {
      //this.body.insertBefore(
        //h('div', { innerHTML: `<b>${name}</b> ${event} ${amount}LP` }),
        //this.body.firstChild) }
    //else {
      //this.body.insertBefore(
        //h('div', { innerHTML: `<b>${name}</b> ${event}` }),
        //this.body.firstChild) } } }
const log = new History()

// the rewards contract and its participants -------------------------------------------------------
class Pool {

  // in reward token
  interval    = FUND_INTERVAL
  portion     = 2500
  remaining   = 120
  balance     = this.portion

  // in lp token
  last_update = 0
  lifetime    = 0
  locked      = 0

  update () {
    log.now.textContent = `T=${T}`
    log.balance.textContent = `reward budget: ${this.balance.toFixed(3)}`
    log.remaining.textContent = `${this.remaining} days remaining`
    this.lifetime += this.locked
    if (T % this.interval == 0) {
      console.info('fund', this.portion, this.remaining)
      if (this.remaining > 0) {
        this.balance += this.portion
        this.remaining -= 1 } } } }
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
    stacked.add(this) }
    //lifetime.add(this)
    //earned.add(this) }
  retrieve (amount: number) {
    if (this.locked < amount) return
    this.last_update = T
    this.locked -= amount
    log.add('retrieves', this.name, amount)
    if (this.locked === 0) current.remove(this) }
  claim () {
    if (this.locked === 0)
      return
    if (this.age < THRESHOLD)
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
    //claimed.add(this)
    if (RESET_AGE) this.age = 0 }
  update () { // WARNING assumes elapsed=1 !
    this.lifetime += this.locked
    if (this.locked > 0) this.age++
    this.earned = pool.balance * this.lifetime / pool.lifetime
    this.claimable = this.earned - this.claimed
    if (NO_TABLE) return
    table.rows[this.name].last_update.textContent = String(this.last_update)
    table.rows[this.name].locked.textContent      = String(this.locked)
    table.rows[this.name].lifetime.textContent    = String(this.lifetime)
    table.rows[this.name].age.textContent         = String(this.age)
    table.rows[this.name].earned.textContent      = this.earned.toFixed(3)
    table.rows[this.name].claimed.textContent     = this.claimed.toFixed(3)
    table.rows[this.name].claimable.textContent   = this.claimable.toFixed(3)
    const [fill, stroke] = this.colors()
    table.rows[this.name].claimable.style.backgroundColor = fill
    table.rows[this.name].claimable.style.color           = stroke }
  colors () {
    switch (true) {
      case this.claimable > pool.balance:
        return [Gruvbox.fadedRed,    Gruvbox.brightRed]
      case this.claimed > this.earned:
        return [Gruvbox.fadedOrange, Gruvbox.brightOrange]
      case this.age == THRESHOLD:
        return [Gruvbox.brightAqua,  Gruvbox.brightAqua]
      case this.age >  THRESHOLD:
        return [Gruvbox.fadedAqua,   Gruvbox.brightAqua]
      default:
        return [Gruvbox.dark0,       Gruvbox.light0] } } }

type Users  = Record<string, User>

const users: Users = {}

for (let i = 0; i < MAX_USERS; i++) {
  const name    = `User${i}`
  const balance = Math.floor(Math.random()*MAX_INITIAL)
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
class Table {
  root: HTMLElement;
  rows: Rows = {};
  constructor () {
    this.root = document.createElement('table')
    if (NO_TABLE) return
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
    if (NO_TABLE) return
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
class PieChart {
  root:   HTMLElement;
  title:  HTMLElement;
  label:  HTMLElement;
  canvas: HTMLCanvasElement;

  users: Users = {};
  total: number = 0;
  field: string;

  constructor (name: string, field: string) {
    this.field  = field
    this.root   = h('div', { className: `pie ${field}` })
    this.title  = addTo(this.root, h('h1', { textContent: name }))
    this.label  = addTo(this.root, h('h2'))
    this.canvas = addTo(this.root, h('canvas', { width: 1, height: 1 })) as HTMLCanvasElement }

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
    this.label.textContent = total.toFixed(3)
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
        context.strokeStyle = fillStyle//strokeStyle
        context.fill()
        context.stroke()
        start = end } } } }

class StackedPieChart {
  root:   HTMLElement;
  //title:  HTMLElement;
  //label:  HTMLElement;
  canvas: HTMLCanvasElement;

  users: Users = {};
  add (user: User) {
    this.users[user.name] = user }
  remove (user: User) {
    delete this.users[user.name] }

  constructor () {
    this.root   = h('div', { className: `pie stacked` })
    //this.title  = addTo(this.root, h('h1', { textContent: name }))
    //this.label  = addTo(this.root, h('h2'))
    this.canvas = addTo(this.root, h('canvas', { width: 1, height: 1 })) as HTMLCanvasElement }

  resize () {
    this.canvas.width = this.canvas.height = 1
    const size = Math.min(this.root.offsetWidth, this.root.offsetHeight)
    this.canvas.width = this.canvas.height = size
    this.render() }

  render () {
    // extract needed datum from user list
    // and sum the total
    let total: number = 0
    for (const user of Object.values(this.users)) {
      total += user.lifetime }
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
      const [fillStyle, strokeStyle] = users[name].colors()
      context.fillStyle = fillStyle
      context.strokeStyle = fillStyle//strokeStyle
      context.fill()
      context.stroke()
      start = end } } }

const current = new PieChart('Current amounts locked',  'locked')
const stacked = new StackedPieChart()
//const lifetime  = new PieChart('Lifetime', 'lifetime')
//const earned    = new PieChart('Earned',   'earned')
//const claimed   = new PieChart('Claimed',  'claimed')

;(function start () {
  // add components
  for (const el of [ current
                   , stacked
                   //, lifetime
                   //, earned
                   //, claimed
                   , log
                   , table /*,sparkline*/]) {
    addTo(document.body, el.root) }

  resize()
  window.addEventListener('resize', throttle(100, resize))

  update()
  function update () {
    // advance time
    T++

    // periodically fund pool and increment its lifetime
    pool.update()

    // increment lifetimes and ages; collect eligible claimants
    const eligible: Array<User> = []
    for (const user of Object.values(users)) {
      user.update()
      if (user.claimable > 0) eligible.push(user)
    }

    // perform random lock/retrieve from random account for random amount
    const user   = pickRandom(Object.values(users))//Object.keys(users)[Math.floor(Math.random()*Object.keys(users).length)]
    const action = pickRandom([(amount:number)=>user.lock(amount)
                              ,(amount:number)=>user.retrieve(amount)])//Object.values(actions)[random(actions.length)]
    action(random(user.balance))

    // perform random claim
    if (eligible.length > 0) {
      const claimant = pickRandom(eligible)
      claimant.claim()
    }

    // update charts
    for (const chart of [current
                        ,stacked
                        //,lifetime
                        //,earned
                        //,claimed
                        ]) { chart.render() }

    // rinse and repeat
    setTimeout(update, UPDATE_INTERVAL) }
  function resize () {
    current.resize()
    stacked.resize()
    //lifetime.resize()
    //earned.resize()
    //claimed.resize()
  } })()

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

import './style.css'

import { EventEmitter } from 'events'

// pool globals
const threshold    = 24
const fundInterval = 24
const fundPortion  = 2500
let   poolBalance  = 2500

// generate test accounts
const numberOfAccounts = 50
interface Account {
  name:     string
  balance:  number
  locked:   number
  unlocked: number
  claimed:  number
  age:      number
}
const accounts: Record<string, Account> = {}
for (let i = 0; i < numberOfAccounts; i++) {
  const name = `User${i}`
  accounts[name] = {
    name,
    balance: Math.floor(Math.random()*1000000),
    locked:   0,
    unlocked: 0,
    claimed:  0,
    age:      0,
  }
}

// dom helpers
function h (element: string, attributes={}) {
  return Object.assign(document.createElement(element), attributes) }
function addTo (parent: HTMLElement, child: HTMLElement) {
  return parent.appendChild(child) }

type PieChartValues = Record<string, number>;
export class PieChart extends EventEmitter {

  _values: PieChartValues = {}
  get values () {
    return this._values }
  set values (v: PieChartValues) {
    this._values = v
    this.render() }
  get total () {
    return Object.values(this.values)
      .reduce((total, datum)=>total+datum, 0) }

  root:   HTMLElement;
  title:  HTMLElement;
  label:  HTMLElement;
  canvas: HTMLCanvasElement;
  list:   HTMLElement;

  constructor (name: string, className: string) {
    super()
    this.root   = h('div', { className })
    this.title  = addTo(this.root, h('h1', { textContent: name }))
    this.label  = addTo(this.root, h('h2'))
    this.canvas = addTo(this.root, h('canvas', { width: 1, height: 1 })) as HTMLCanvasElement
    this.list   = addTo(this.root, h('ol')) }

  resize () {
    this.canvas.width = this.canvas.height = 1
    const size = Math.min(this.root.offsetWidth, this.root.offsetHeight)
    this.canvas.width = this.canvas.height = size
    this.render() }

  render () {
    const total = this.total
    this.label.textContent = String(total)
    const {width, height} = this.canvas
    const context = this.canvas.getContext('2d') as CanvasRenderingContext2D;
    // clear
    this.list.innerHTML = ''
    context.fillStyle = '#282828'
    context.fillRect(1, 1, width-2, height-2)
    // define center
    const centerX = width  / 2
    const centerY = height / 2
    const radius  = centerX * 0.95
    // loop over segments
    const values = Object.entries(this.values)
      .sort(([_1,datum1],[_2,datum2])=>datum2-datum1)
    let start = 0
    for (const [name, datum] of values) {
      this.list.innerHTML += `<li><b>${name}</b> -> ${datum}</li>`
      const portion = datum/total
      const end     = start + (2*portion)
      context.beginPath()
      context.moveTo(centerX, centerY)
      context.arc(centerX, centerY, radius, start * Math.PI, end * Math.PI)
      //context.fillStyle = '#444'
      //context.strokeStyle = '#ebdbb2'
      if (accounts[name].age == threshold) {
        context.fillStyle   = '#b8bb26'
        context.strokeStyle = '#b8bb26'
      } else if (accounts[name].age > threshold) {
        context.fillStyle   = '#98971a'
        context.strokeStyle = '#b8bb26'
      } else {
        context.fillStyle   = '#d79921'
        context.strokeStyle = '#fabd2f'
      }
      if (accounts[name].claimed > accounts[name].unlocked) {
        context.fillStyle   = '#cc241d'
        context.strokeStyle = '#fb4934'
      }
      context.fill()
      context.stroke()
      start = end
    } }

  increment (name: string, amount: number) {
    this.values = {...this.values, [name]: (this.values[name]||0) + amount }
  }

  decrement (name: string, amount: number) {
    const value = (this.values[name]||0)
    if (value < amount) return
    this.values = {...this.values, [name]: value - amount }
  }
}

export class History extends EventEmitter {
  root        = h('div', { className: 'history' })
  startButton = addTo(this.root, h('button', { textContent: 'start', onclick: this.start.bind(this) }))
  body        = addTo(this.root, h('ol'))

  start () {}

  add (event: string, name: string, amount: number|undefined) {
    if (amount) {
      this.body.innerHTML = `<div><b>${name}</b> ${event} ${amount} LP</div>` + this.body.innerHTML }
    else {
      this.body.innerHTML = `<div><b>${name}</b> ${event}</div>` + this.body.innerHTML } }
}

export class Sparkline extends EventEmitter {
  root   = h('div', { className: 'sparkline' })
  canvas = addTo(this.root, h('canvas'))

  tick () {}
  fund () {}
}

main()

function main () {
  const current   = new PieChart('Current',  'pie locked')
  const lifetime  = new PieChart('Lifetime', 'pie lifetime')
  const unlocked  = new PieChart('Unlocked', 'pie unlocked')
  const claimed   = new PieChart('Claimed',  'pie claimed')
  const history   = new History()
  const sparkline = new Sparkline()

  document.body.appendChild(current.root)
  document.body.appendChild(lifetime.root)
  document.body.appendChild(unlocked.root)
  document.body.appendChild(claimed.root)
  document.body.appendChild(history.root)
  document.body.appendChild(sparkline.root)

  const actions = {
    lock   (account: Account, amount: number) {
      account.locked += amount
      history.add('locks', account.name, amount)
      current.increment(account.name, amount)
    },
    unlock (account: Account, amount: number) {
      if (account.locked < amount) return
      history.add('unlocks', account.name, amount)
      current.decrement(account.name, amount)
    },
    claim  (account: Account) {
      if (account.locked == 0) return
      if (account.age < threshold) return
      console.log('claim', account.name, claimed.values[account.name], unlocked.values[account.name])
      if ((claimed.values[account.name]||0) < unlocked.values[account.name]) {
        history.add('claim', account.name, undefined)
        account.claimed = unlocked.values[account.name]
        claimed.values = {...claimed.values, [account.name]: unlocked.values[account.name] }
      }
    }
  }

  let T = 0
  step()
  function step () {
    T++
    if (T % fundInterval == 0) poolBalance += fundPortion

    const action  = Object.values(actions)[Math.floor(Math.random()*3)]
    const name    = Object.keys(accounts)[Math.floor(Math.random()*Object.keys(accounts).length)]
    const account = accounts[name]
    const amount  = Math.floor(Math.random()*account.balance)
    for (const [name, amount] of Object.entries(current.values)) {
      lifetime.increment(name, amount)
      if (amount > 0) accounts[name].age += 1
    }
    for (const [name, amount] of Object.entries(lifetime.values)) {
      unlocked.values = { ...unlocked.values, [name]: poolBalance * amount / lifetime.total }
      accounts[name].unlocked = poolBalance * amount / lifetime.total
    }
    action(account, amount)
    //current.values  = [...Array(Math.floor(Math.random()*1000))].map(()=>Math.floor(Math.random()*1000)).sort()
    //lifetime.values = [...Array(Math.floor(Math.random()*1000))].map(()=>Math.floor(Math.random()*1000)).sort()
    //unlocked.values = [...Array(Math.floor(Math.random()*1000))].map(()=>Math.floor(Math.random()*1000)).sort()
    //claimed.values  = [...Array(Math.floor(Math.random()*1000))].map(()=>Math.floor(Math.random()*1000)).sort()
    setTimeout(step, 100)
  }

  resize()
  window.addEventListener('resize', throttle(100, resize))
  function resize () {
    current.resize()
    lifetime.resize()
    unlocked.resize()
    claimed.resize()
  }
}

function throttle (t: number, fn: Function) {
  // todo replacing t with a function allows for implementing exponential backoff
  let timeout: any
  return function throttled (...args:any) {
    return new Promise(resolve=>{
      if (timeout) clearTimeout(timeout)
      timeout = after(t, ()=>resolve(fn(...args)))
      //console.debug('vvvvVVVVV', t, fn)
    })
  }
}

function after (t: number, fn: Function) {
  return setTimeout(fn, t)
}

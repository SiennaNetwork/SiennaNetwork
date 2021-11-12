import './style.css'

import {
  h, random, pickRandom, throttle, after, append, prepend, encode, decode
} from './helpers'

// settings ----------------------------------------------------------------------------------------
export const TIME_SCALE          = 864
           , EPOCHS              = 14
           , DIGITS              = 1000
           , DIGITS_INV          = Math.log10(DIGITS)
           , FUND_PORTION        = 2500 * DIGITS
           , FUND_INTERVAL       = 86400/TIME_SCALE
           , COOLDOWN            = FUND_INTERVAL
           , THRESHOLD           = FUND_INTERVAL
           , USER_GIVES_UP_AFTER = Infinity
           , MAX_USERS           = 10
           , MAX_INITIAL         = 10000
           , UPDATE_INTERVAL     = 20
           , AUTO_CLAIM          = false
           , AUTO_LOCK_UNLOCK    = false

// colors ------------------------------------------------------------------------------------------
import Gruvbox from './gruvbox'
const COLORS = Object.assign(
  function getColor (pool: Pool, user: User) {
    switch (true) {
      case user.age < THRESHOLD || user.cooldown > 0: // waiting for age threshold
        return COLORS.COOLDOWN
      case user.claimable > 0 && user.cooldown == 1:  // have rewards to claim
        return COLORS.CLAIMING
      //case user.claimable > 0 && user.cooldown > 0: // just claimed, cooling down
        //return COLORS.ALL_OK
      case user.claimable > pool.balance:             // not enough money in pool
        return COLORS.BLOCKED 
      case user.claimed > user.earned:                // crowded out
        return COLORS.CROWDED
      case user.claimable === 0:
        return COLORS.NOTHING
      default:
        return COLORS.CLAIMABLE
    }
  }, {
    CLAIMABLE: [Gruvbox.fadedAqua,   Gruvbox.brightAqua],
    CLAIMING:  [Gruvbox.brightAqua,  Gruvbox.brightAqua],
    BLOCKED:   [Gruvbox.fadedOrange, Gruvbox.brightOrange],
    CROWDED:   [Gruvbox.fadedPurple, Gruvbox.brightPurple],
    COOLDOWN:  [Gruvbox.fadedBlue,   Gruvbox.brightBlue],
    NOTHING:   [Gruvbox.dark0,       Gruvbox.brightYellow]
  }
)

import initRewards, * as Bound from '../target/web/rewards.js'

document.body.innerHTML = '<center>loading</center>'

// load then start on click ------------------------------------------------------------------------
initReal().then(()=>{ 
  document.body.onclick = () => {
    document.body.innerHTML = ''
    document.body.onclick = null
    start() }
  document.body.innerHTML = '<center>click to start</center>'
})

// wasm module load & init -------------------------------------------------------------------------
export default async function initReal () {
  // thankfully wasm-pack/wasm-bindgen left an escape hatch
  // because idk wtf is going on with the default loading code
  const url = new URL('rewards_bg.wasm', location.href)
      , res = await fetch(url.toString())
      , buf = await res.arrayBuffer()
  await initRewards(buf)
}

function start () {

  // create the dashboard --------------------------------------------------------------------------
  const ui = {
    log:     new Sidebar(),
    table:   new Table(),
    current: new PieChart('Current amounts locked',  'locked'),
    stacked: new StackedPieChart()
  }

  // create a pool and some of test users with random balances -------------------------------------
  const pool = new Pool(ui)
  const users: Users = {}
  for (let i = 0; i < MAX_USERS; i++) {
    const name    = `User${i}`
    const balance = Math.floor(Math.random()*MAX_INITIAL)
    users[name]   = new RealUser(ui, pool, name, balance)
  }

  // add components --------------------------------------------------------------------------------
  for (const el of Object.values(ui)) {
    append(document.body, el.root)
  }

  // create dom elements for all users - then only update the content ------------------------------
  ui.table.init(users)

  // add resize handler ----------------------------------------------------------------------------
  resize()
  window.addEventListener('resize', throttle(100, resize))

  // start updating --------------------------------------------------------------------------------
  update()
  function update () {
    try {
      // advance time --------------------------------------------------------------------------------
      T.T++
      pool.contract.block = T.T
      pool.contract.time  = T.T

      // periodically fund pool and increment its lifetime -------------------------------------------
      pool.update()

      // increment lifetimes and ages; collect eligible claimants ------------------------------------
      const eligible: Array<User> = []
      for (const user of Object.values(users)) {
        user.update()
        if (user.earned > 0) eligible.push(user as User)
      }

      // perform random lock/retrieve from random account for random amount --------------------------
      if (AUTO_LOCK_UNLOCK) {
        const user = pickRandom(Object.values(users))
        pickRandom([
          (amount:number)=>user.lock(amount),
          (amount:number)=>user.retrieve(amount)
        ])(random(user.balance))
      }

      // perform random claim ------------------------------------------------------------------------
      if (AUTO_CLAIM && eligible.length > 0) {
        const claimant = pickRandom(eligible)
        claimant.claim()
      }

      // update charts -------------------------------------------------------------------------------
      for (const chart of [ui.current,ui.stacked]) {
        chart.render()
      }
    } catch (e) {
      console.error(e)
    }
    // rinse and repeat ----------------------------------------------------------------------------
    after(UPDATE_INTERVAL, update)
  }

  // resize handler --------------------------------------------------------------------------------
  function resize () {
    ui.current.resize()
    ui.stacked.resize()
  }
}

export const format = {
  integer:    (x:number) => String(x),
  decimal:    (x:number) => (x/DIGITS).toFixed(DIGITS_INV),
  percentage: (x:number) => `${format.decimal(x)}%`
}

// root of time (warning, singleton!) --------------------------------------------------------------
export const T = { T: 1 }

class RPT {
  interval  = FUND_INTERVAL
  portion   = FUND_PORTION
  remaining = EPOCHS
  vest (pool: Pool) {
    if (T.T % this.interval == 0) {
      console.info('fund', this.portion, this.remaining)
      if (this.remaining > 0) {
        this.portion
        this.remaining -= 1
        pool.epoch++
        pool.contract.sender = ""
        pool.contract.handle({rewards:{begin_epoch:{next_epoch:pool.epoch}}})
        return this.portion
      }
    }
    return 0
  }
}

export class Pool {

  contract: Rewards = new Rewards()
  rpt:      RPT     = new RPT()

  ui:          UIContext

  last_update: number = 0
  lifetime:    number = 0
  locked:      number = 0
  claimed:     number = 0
  cooldown:    number = 0
  threshold:   number = 0
  liquid:      number = 0

  epoch: number = 0

  balance:     number = this.rpt.vest(this)

  constructor (ui: UIContext) {
    this.ui = ui
    this.contract.init({
      config: {
        reward_token: { address: "SIENNA",  code_hash: "" },
        lp_token:     { address: "LPTOKEN", code_hash: "" },
        viewing_key:  "",
        bonding:      COOLDOWN,
      }
    })
    this.ui.log.close.onclick = this.close.bind(this)
  }

  update () {

    let portion = this.rpt.vest(this)
    this.balance += portion

    this.contract.next_query_response = {balance:{amount:String(this.balance)}}

    const {
      rewards:{pool_info:{updated, volume, staked, distributed, bonding, budget, clock}}
    } = this.contract.query({rewards:{pool_info:{at:T.T}}})

    Object.assign(this, {
      last_update: updated,
      lifetime:    Number(volume),
      locked:      Number(staked),
      claimed:     Number(distributed),
      threshold:   bonding,
      cooldown:    bonding,
      balance:     Number(budget)
    })

    this.ui.log.now.setValue(T.T)
    this.ui.log.epoch.setValue(`${clock.number}/${EPOCHS}`)
    this.ui.log.epoch_started.setValue(clock.started)
    this.ui.log.epoch_start_volume.setValue(clock.volume)

    this.ui.log.lifetime.setValue(this.lifetime)
    this.ui.log.locked.setValue(this.locked)

    this.ui.log.balance.setValue(format.decimal(this.balance))
    this.ui.log.claimed.setValue(format.decimal(this.claimed))

    this.ui.log.cooldown.setValue(this.cooldown)
    this.ui.log.threshold.setValue(this.threshold)
    this.ui.log.liquid.setValue(format.percentage(this.liquid))
  }

  close () {
    this.contract.sender = ""
    this.contract.handle({rewards:{close:{message:"pool closed"}}})
  }
}
////////////////////////////////////////////////////////////////////////////////////////////////////
export class User {
  ui:           UIContext
  pool:         Pool
  name:         string
  balance:      number
  last_update:  number = 0
  lifetime:     number = 0
  locked:       number = 0
  age:          number = 0
  earned:       number = 0
  claimed:      number = 0
  claimable:    number = 0
  cooldown:     number = 0
  waited:       number = 0
  last_claimed: number = 0
  share:        number = 0

  pool_volume_at_entry:    number = 0
  pool_volume_since_entry: number = 0
  rewards_since_entry:     number = 0

  constructor (ui: UIContext, pool: Pool, name: string, balance: number) {
    this.ui      = ui
    this.pool    = pool
    this.name    = name
    this.balance = balance
  }
  update () {
    this.ui.table.update(this)
  }
  lock (amount: number) {
    this.ui.log.add('locks', this.name, amount)
    this.ui.current.add(this)
    this.ui.stacked.add(this)
  }
  retrieve (amount: number) {
    this.ui.log.add('retrieves', this.name, amount)
    if (this.locked === 0) this.ui.current.remove(this)
  }
  claim () {
    throw new Error('not implemented')
  }
  doClaim (reward: number) { // stupid typescript inheritance constraints
    if (reward <= 0) return 0

    if (this.locked === 0) return 0

    if (this.cooldown > 0) return 0

    if (this.claimed > this.earned) {
      this.ui.log.add('crowded out A', this.name, undefined)
      return 0
    }

    if (reward > this.pool.balance) {
      this.ui.log.add('crowded out B', this.name, undefined)
      return 0
    }

    this.pool.balance -= reward
    this.ui.log.add('claim', this.name, reward)
    console.debug('claimed:', reward)
    console.debug('remaining balance:', this.pool.balance)
    return reward
  }

  colors () {
    return COLORS(this.pool, this)
  }
}
export class RealUser extends User {
  address: string
  get contract () {
    return (this.pool as Pool).contract
  }
  constructor (ui: UIContext, pool: Pool, name: string, balance: number) {
    super(ui, pool, name, balance)
    this.address = this.name
    this.contract.sender = this.address
    this.contract.handle({ set_viewing_key: { key: "" } })
  }
  update () {
    // mock the user's balance - actually stored on this same object
    // because we don't have a snip20 contract to maintain it
    this.contract.next_query_response = {balance:{amount:String(this.pool.balance)}}
    // get the user's info as stored and calculated by the rewards contract
    // presuming the above mock balance
    const {rewards:{user_info:{
      updated, volume, staked, earned,
      starting_pool_volume, accumulated_pool_volume, accumulated_pool_rewards,
      bonding
    }}} = this.contract.query({
      rewards:{user_info:{at:T.T,address:this.address,key:""}}
    });
    Object.assign(this, {
      last_update:             updated,
      lifetime:                Number(volume),
      locked:                  Number(staked),
      earned:                  Number(earned),
      pool_volume_at_entry:    Number(starting_pool_volume),
      pool_volume_since_entry: Number(accumulated_pool_volume),
      rewards_since_entry:     Number(accumulated_pool_rewards),
      share:                   Number(volume) / Number(accumulated_pool_volume) * 100000,
      cooldown:                Number(bonding)
    })
    super.update()
  }
  lock (amount: number) {
    this.contract.sender = this.address
    try {
      //console.debug('lock', amount)
      const msg = {rewards:{lock:{amount: String(amount)}}};
      this.contract.handle(msg)
      super.lock(amount) }
    catch (e) {
      //console.error(e)
    }
  }
  retrieve (amount: number) {
    this.contract.sender = this.address
    try {
      //console.debug('retrieve', amount)
      const msg = {rewards:{retrieve:{amount: String(amount)}}};
      this.contract.handle(msg)
      super.retrieve(amount)
    } catch (e) {
      //console.error(e)
    }
  }
  claim () {
    this.contract.sender = this.address
    try {
      const result = this.contract.handle({ rewards: { claim: {} } })
      const reward = Number(result.log.reward)
      return this.doClaim(reward)
    } catch (e) {
      console.error(e)
      return 0
    }
  }
}
export type Users = Record<string, User>
// wrapper classes on the js side too... -----------------------------------------------------------
interface LogAttribute {
  key:   string,
  value: string
}
interface HandleResponse {
  messages: Array<object>,
  log:      any,
  data:     any
}
class Rewards {
  index = 0
  contract = new Bound.Contract()
  debug = false
  init (msg: object) {
    this.index += 1
    this.block = T.T
    if (this.debug) console.debug(`init> ${this.index}`, msg)
    const res = decode(this.contract.init(encode(msg)))
    if (this.debug) console.debug(`<init ${this.index}`, res)
    return res
  }
  query (msg: object) {
    this.index += 1
    this.block = T.T
    if (this.debug) console.debug(`query> ${this.index}`, msg)
    const res = decode(this.contract.query(encode(msg)))
    if (this.debug) console.debug(`<query ${this.index}`, res)
    if (res.pool_info) console.log(this.index, res.pool_info)
    return res
  }
  handle (msg: object) {
    this.index += 1
    this.block = T.T
    if (this.debug) console.debug(`handle> ${this.index}`, msg)
    const res: HandleResponse = decode(this.contract.handle(encode(msg)))
    res.log = Object.fromEntries(Object
      .values(res.log as object)
      .map(({key, value})=>[key, value]))
    if (Object.keys(res.log).length > 0) console.log(res.log)
    if (this.debug) console.debug(`<handle ${this.index}`, res)
    return res
  }
  set next_query_response (response: object) {
    this.contract.next_query_response = encode(response)
  }
  set sender (address: string) {
    this.contract.sender = encode(address)
  }
  set block (height: number) {
    this.contract.block = BigInt(height)
  }
  set time (time: number) {
    this.contract.time = BigInt(time)
  }
}

// killswitches for gui components -----------------------------------------------------------------
export const NO_HISTORY = true
export const NO_TABLE   = false

// handles to dashboard components that can be passed into User/Pool objects -----------------------
// normally we'd do this with events but this way is simpler
export interface UIContext {
  log:     Sidebar
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
export class Sidebar {
  root      = h('div', { className: 'history' })
  body      = append(this.root, h('ol'))

  now                = new Field('time').append(this.root)
  epoch              = new Field('epoch').append(this.root)
  epoch_started      = new Field('epoch started @').append(this.root)
  epoch_start_volume = new Field('pool volume @ epoch start').append(this.root)

  locked    = new Field('liquidity now in pool').append(this.root)
  lifetime  = new Field('all liquidity ever in pool').append(this.root)

  balance   = new Field('available reward balance').append(this.root)
  claimed   = new Field('rewards claimed by users').append(this.root)

  threshold = new Field('initial age threshold').append(this.root)
  cooldown  = new Field('cooldown after claim').append(this.root)
  liquid    = new Field('pool liquidity ratio').append(this.root)

  close = append(this.root, h('button', { textContent: 'close pool' }))

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
  // fields
  name:                HTMLElement
  last_update:         HTMLElement
  volume_at_entry:     HTMLElement
  locked:              HTMLElement

  lifetime:            HTMLElement
  sign1: HTMLElement
  volume_since_entry:  HTMLElement
  sign2: HTMLElement
  share:               HTMLElement
  sign3: HTMLElement
  rewards_since_entry: HTMLElement
  sign4: HTMLElement
  earned:              HTMLElement

  age:                 HTMLElement
  claimed:             HTMLElement
  claimable:           HTMLElement
  cooldown:            HTMLElement

  // buttons
  lockedMinus100:      HTMLElement
  lockedMinus1:        HTMLElement
  lockedValue:         HTMLElement
  lockedPlus1:         HTMLElement
  lockedPlus100:       HTMLElement
}

type Rows = Record<string, Columns>

export class Table {
  root: HTMLElement;
  rows: Rows = {};

  constructor () {
    this.root = document.createElement('table')
    if (NO_TABLE) return }

  init (users: Users) {
    append(this.root, h('thead', {},
      h('th', { textContent: 'name' }),
      h('th', { innerHTML:   'last<br>update' }),
      h('th', { innerHTML:   'pool volume<br>@ entry epoch'  }),
      h('th', { textContent: 'current stake' }),
      h('th', { innerHTML:   'liquidity<br>contribution' }),
      h('th', { textContent: '÷' }),
      h('th', { innerHTML:   'pool volume<br>since entry epoch<br>or last claim'  }),
      h('th', { textContent: '=' }),
      h('th', { textContent: 'share' }),
      h('th', { textContent: '×' }),
      h('th', { innerHTML:   'rewards vested<br>since entry' }),
      h('th', { textContent: '=' }),
      h('th', { textContent: 'earned' }),
      h('th', { textContent: 'cooldown' }),
    ))
    for (const name of Object.keys(users)) {
      this.addRow(name, users[name])
    }
  }

  addRow (name: string, user: User) {
    if (NO_TABLE) return
    const row = append(this.root, h('tr'))
    const locked =
            h('td', { className: 'locked' })
        , lockedMinus100 = append(locked, h('button', { textContent: '-100', onclick: () => user.retrieve(100) }))
        , lockedMinus1   = append(locked, h('button', { textContent:   '-1', onclick: () => user.retrieve(1) }))
        , lockedValue    = append(locked, h('span',   { textContent:    '' }))
        , lockedPlus1    = append(locked, h('button', { textContent:   '+1', onclick: () => user.lock(1) }))
        , lockedPlus100  = append(locked, h('button', { textContent: '+100', onclick: () => user.lock(100) }))
    const fields = this.rows[name] = {
      name:                append(row, h('td', { style: 'font-weight:bold', textContent: name })),
      last_update:         append(row, h('td')),
      volume_at_entry:     append(row, h('td')),
      locked:              append(row, locked),
      lockedMinus100, lockedMinus1, lockedValue, lockedPlus1, lockedPlus100,
      age:                 /*append(row, */h('td')/*)*/,
      lifetime:            append(row, h('td')),
      sign1:               append(row, h('td', { textContent: '÷' })),
      volume_since_entry:  append(row, h('td')),
      sign2:               append(row, h('td', { textContent: '=' })),
      share:               append(row, h('td')),
      sign3:               append(row, h('td', { textContent: '×' })),
      rewards_since_entry: append(row, h('td')),
      sign4:               append(row, h('td', { textContent: '=' })),
      earned:              append(row, h('td', { className: 'claimable', onclick: () => {user.claim()} })),
      claimed:             /*append(row,*/ h('td')/*)*/,
      claimable:           /*append(row,*/ h('td', { className: 'claimable', onclick: () => {user.claim()} })/*)*/,
      cooldown:            append(row, h('td')),
    }
    fields.share.style.fontWeight = 'bold'
    fields.claimable.style.fontWeight = 'bold'
    append(this.root, row)
    return fields
  }

  update (user: User) {
    if (NO_TABLE) return
    this.rows[user.name].last_update.textContent =
      format.integer(user.last_update)
    this.rows[user.name].lockedValue.textContent =
      format.integer(user.locked)
    this.rows[user.name].lifetime.textContent =
      format.integer(user.lifetime)
    this.rows[user.name].volume_at_entry.textContent =
      format.integer(user.pool_volume_at_entry)
    this.rows[user.name].volume_since_entry.textContent =
      format.integer(user.pool_volume_since_entry)
    this.rows[user.name].rewards_since_entry.textContent =
      format.integer(user.rewards_since_entry)
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
      const centerX = width   / 2
      const centerY = height  / 2
      const radius  = centerX * 0.95

      // loop over segments
      let start = 0
      for (const name of Object.keys(this.users).sort()) {
        const value = values[name]
        if (!value) continue
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
        start = end
      }
    })
  }
}

export class StackedPieChart {
  root:   HTMLElement;
  canvas: HTMLCanvasElement;
  users:  Users = {};

  add (user: User) {
    this.users[user.name] = user
  }
  remove (user: User) {
    delete this.users[user.name]
  }
  constructor () {
    this.root   = h('div', { className: `pie stacked` })
    this.canvas = append(this.root, h('canvas', { width: 1, height: 1 })) as HTMLCanvasElement
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
      const centerX = width   / 2
      const centerY = height  / 2
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
        start = end
      }
    })
  }
}

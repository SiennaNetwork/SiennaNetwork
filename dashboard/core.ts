import { h, append, format } from './helpers'

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

export class Dashboard {
  root         = document.body
  environment  = new Environment()
  sienna       = new SNIP20('SIENNA')
  mgmt         = new MGMT()
  rpt          = new RPT()
  microservice = new Microservice()
  lpToken      = new SNIP20('LP_XXX_YYY')
  rewards_v3   = new RewardPool('v3')
  migrate      = h('button', { textContent: 'migrate' })
  rewards_v4   = new RewardPool('v4')

  constructor () {
    //this.root.innerHTML = '<center>loading</center>'
    for (const el of [
      this.environment,
      this.sienna, this.mgmt, this.rpt, this.microservice,
      this.rewards_v3, this.rewards_v4, this.lpToken,
    ]) {
      append(this.root, el.root)
    }

    this.sienna.add('Admin')
    this.lpToken.add('Admin')

    for (let i = 0; i < 10; i++) {
      const id = `User${i}`
      this.sienna.add(id)
      this.lpToken.add(id, Math.floor(Math.random()*100000))
      this.rewards_v3.add(id)
      this.rewards_v4.add(id)
    }

    this.sienna.add('Rewards V3')
    this.sienna.add('Rewards V4')
    this.sienna.add('MGMT')
    this.sienna.add('RPT')

    this.lpToken.add('Rewards V3')
    this.lpToken.add('Rewards V4')
  }
}

type Timer = ReturnType<typeof setTimeout>

export class Environment {
  root  = h('section', { className: 'Module Environment' })
  title = append(this.root, h('header', { textContent: 'Environment' }))

  time = 0
  rate = 1
  timer: Timer|null = null

  timeDisplay = new Field(this.root, "Time", this.time)

  start () {
    this.timer = setInterval(this.update.bind(this), this.rate)
  }

  pause () {
    if (this.timer) clearInterval(this.timer)
    this.timer = null
  }

  update () {
    this.time += this.rate
  }
}

export class SNIP20 {
  root  = h('section', { className: 'Module SNIP20', })
  title = append(this.root, h('header', { textContent: 'SNIP20' }))
  table = append(this.root, h('table'))

  constructor (id: string) {
    this.title.textContent = id
    this.root.classList.add(id)
  }

  balances: Record<string, number> = {}
  displays: Record<string, Field>  = {}
  add (id: string, balance: number = 0) {
    this.balances[id] = balance
    this.displays[id] = new Field(this.root, id, balance)
  }
}

export class Microservice {
  root  = h('section', { className: 'Module Microservice' })
  title = append(this.root, h('header', { textContent: 'Microservice' }))

  epoch = 0
  epochDisplay = new Field(this.root, "Epoch", this.epoch)
}

export class MGMT {
  root  = h('section', { className: 'Module MGMT' })
  title = append(this.root, h('header', { textContent: 'MGMT' }))

  portion = 2500
}

export class RPT {
  root  = h('section', { className: 'Module RPT' })
  title = append(this.root, h('header', { textContent: 'RPT' }))

  portion = 2500
}

export class RewardPool {
  root  = h('section', { className: 'Module Rewards' })
  title = append(this.root, h('header', { textContent: 'Rewards' }))

  stakedPie = new PieChart()
  volumePie = new PieChart()

  closed: [number, string] | null = null
  staked:      number = 0
  volume:      number = 0
  updated:     number = 0
  bonding:     number = 0
  unlocked:    number = 0
  distributed: number = 0
  budget:      number = 0

  constructor (id: string) {
    this.title.textContent = `Rewards ${id}`
    this.root.classList.add(id)
    append(this.root, this.stakedPie.root)
    append(this.root, this.volumePie.root)
  }

  totals: Record<string, Field> = {}
  users:  Record<string, User> = {}
  add (id: string) {
    this.users[id] = new User(this.root, id)
  }
}

export class User {
  root = h('section', { className: 'User' })
  ui = { 
    id:
      new Field(this.root, 'ID', this.id),
    staked:
      new Field(this.root, 'Staked', 0),
    volume:
      new Field(this.root, 'Volume', 0),
    starting_pool_volume:
      new Field(this.root, 'Pool volume at entry',       0),
    accumulated_pool_volume:
      new Field(this.root, 'Pool volume since entry',    0),
    starting_pool_rewards:
      new Field(this.root, 'Reward budget at entry',     0),
    accumulated_pool_rewards:
      new Field(this.root, 'Rewards vested since entry', 0),
    bonding:
      new Field(this.root, 'Remaining bonding period',   0),
    earned:
      new Field(this.root, 'Earned rewards', 0),
  }

  staked:                   number = 0
  pool_share:               number = 0
  volume:                   number = 0
  starting_pool_volume:     number = 0
  accumulated_pool_volume:  number = 0
  reward_share:             number = 0
  starting_pool_rewards:    number = 0
  accumulated_pool_rewards: number = 0
  earned:                   number = 0
  updated:                  number = 0
  elapsed:                  number = 0
  bonding:                  number = 0

  constructor (parent: HTMLElement, public id: string) {
    append(parent, this.root)
    let x = append(this.root, h('div', { className: 'Row' }))
    append(x, this.ui.staked.root)
    append(x, this.ui.volume.root)
    x = append(this.root, h('div', { className: 'Row' }))
    append(x, this.ui.starting_pool_volume.root)
    append(x, this.ui.accumulated_pool_volume.root)
    x = append(this.root, h('div', { className: 'Row' }))
    append(x, this.ui.starting_pool_rewards.root)
    append(x, this.ui.accumulated_pool_rewards.root)
    x = append(this.root, h('div', { className: 'Row' }))
    append(x, this.ui.bonding.root)
    append(x, this.ui.earned.root)
  }
}

export class PieChart {
  root   = h('div', { className: 'pie' })
  canvas = append(this.root, h('canvas', { width: 1, height: 1 }))
}

//export class Pool {

  //contract: Rewards = new Rewards()
  //rpt:      RPT     = new RPT()

  //ui:          UIContext

  //last_update: number = 0
  //lifetime:    number = 0
  //locked:      number = 0
  //claimed:     number = 0
  //cooldown:    number = 0
  //threshold:   number = 0
  //liquid:      number = 0

  //epoch: number = 0

  //balance:     number = this.rpt.vest(this)

  //constructor (ui: UIContext) {
    //this.ui = ui
    //this.contract.init({
      //config: {
        //reward_token: { address: "SIENNA",  code_hash: "" },
        //lp_token:     { address: "LPTOKEN", code_hash: "" },
        //viewing_key:  "",
        //bonding:      COOLDOWN,
      //}
    //})
    //this.ui.log.close.onclick = this.close.bind(this)
  //}

  //update () {

    //let portion = this.rpt.vest(this)
    //this.balance += portion

    //this.contract.next_query_response = {balance:{amount:String(this.balance)}}

    //const {
      //rewards:{pool_info:{updated, volume, staked, distributed, bonding, budget, clock}}
    //} = this.contract.query({rewards:{pool_info:{at:T.T}}})

    //Object.assign(this, {
      //last_update: updated,
      //lifetime:    Number(volume),
      //locked:      Number(staked),
      //claimed:     Number(distributed),
      //threshold:   bonding,
      //cooldown:    bonding,
      //balance:     Number(budget)
    //})

    //this.ui.log.now.setValue(T.T)
    //this.ui.log.epoch.setValue(`${clock.number}/${EPOCHS}`)
    //this.ui.log.epoch_started.setValue(clock.started)
    //this.ui.log.epoch_start_volume.setValue(clock.volume)

    //this.ui.log.lifetime.setValue(this.lifetime)
    //this.ui.log.locked.setValue(this.locked)

    //this.ui.log.balance.setValue(format.decimal(this.balance))
    //this.ui.log.claimed.setValue(format.decimal(this.claimed))

    //this.ui.log.cooldown.setValue(this.cooldown)
    //this.ui.log.threshold.setValue(this.threshold)
    //this.ui.log.liquid.setValue(format.percentage(this.liquid))
  //}

  //close () {
    //this.contract.sender = ""
    //this.contract.handle({rewards:{close:{message:"pool closed"}}})
  //}
//}

//export class BaseUser {
  //ui:           UIContext
  //pool:         Pool
  //name:         string
  //balance:      number
  //last_update:  number = 0
  //lifetime:     number = 0
  //locked:       number = 0
  //age:          number = 0
  //earned:       number = 0
  //claimed:      number = 0
  //claimable:    number = 0
  //cooldown:     number = 0
  //waited:       number = 0
  //last_claimed: number = 0
  //share:        number = 0

  //pool_volume_at_entry:    number = 0
  //pool_volume_since_entry: number = 0
  //rewards_since_entry:     number = 0

  //constructor (ui: UIContext, pool: Pool, name: string, balance: number) {
    //this.ui      = ui
    //this.pool    = pool
    //this.name    = name
    //this.balance = balance
  //}
  //update () {
    //this.ui.table.update(this)
  //}
  //lock (amount: number) {
    //this.ui.log.add('locks', this.name, amount)
    //this.ui.current.add(this)
    //this.ui.stacked.add(this)
  //}
  //retrieve (amount: number) {
    //this.ui.log.add('retrieves', this.name, amount)
    //if (this.locked === 0) this.ui.current.remove(this)
  //}
  //claim () {
    //throw new Error('not implemented')
  //}
  //doClaim (reward: number) { // stupid typescript inheritance constraints
    //if (reward <= 0) return 0

    //if (this.locked === 0) return 0

    //if (this.cooldown > 0) return 0

    //if (this.claimed > this.earned) {
      //this.ui.log.add('crowded out A', this.name, undefined)
      //return 0
    //}

    //if (reward > this.pool.balance) {
      //this.ui.log.add('crowded out B', this.name, undefined)
      //return 0
    //}

    //this.pool.balance -= reward
    //this.ui.log.add('claim', this.name, reward)
    //console.debug('claimed:', reward)
    //console.debug('remaining balance:', this.pool.balance)
    //return reward
  //}

  //colors () {
    //return COLORS(this.pool, this)
  //}
//}

//export class RealUser extends BaseUser {
  //address: string
  //get contract () {
    //return (this.pool as Pool).contract
  //}
  //constructor (ui: UIContext, pool: Pool, name: string, balance: number) {
    //super(ui, pool, name, balance)
    //this.address = this.name
    //this.contract.sender = this.address
    //this.contract.handle({ set_viewing_key: { key: "" } })
  //}
  //update () {
    //// mock the user's balance - actually stored on this same object
    //// because we don't have a snip20 contract to maintain it
    //this.contract.next_query_response = {balance:{amount:String(this.pool.balance)}}
    //// get the user's info as stored and calculated by the rewards contract
    //// presuming the above mock balance
    //const {rewards:{user_info:{
      //updated, volume, staked, earned,
      //starting_pool_volume, accumulated_pool_volume, accumulated_pool_rewards,
      //bonding
    //}}} = this.contract.query({
      //rewards:{user_info:{at:T.T,address:this.address,key:""}}
    //});
    //Object.assign(this, {
      //last_update:             updated,
      //lifetime:                Number(volume),
      //locked:                  Number(staked),
      //earned:                  Number(earned),
      //pool_volume_at_entry:    Number(starting_pool_volume),
      //pool_volume_since_entry: Number(accumulated_pool_volume),
      //rewards_since_entry:     Number(accumulated_pool_rewards),
      //share:                   Number(volume) / Number(accumulated_pool_volume) * 100000,
      //cooldown:                Number(bonding)
    //})
    //super.update()
  //}
  //lock (amount: number) {
    //this.contract.sender = this.address
    //try {
      ////console.debug('lock', amount)
      //const msg = {rewards:{lock:{amount: String(amount)}}};
      //this.contract.handle(msg)
      //super.lock(amount) }
    //catch (e) {
      ////console.error(e)
    //}
  //}
  //retrieve (amount: number) {
    //this.contract.sender = this.address
    //try {
      ////console.debug('retrieve', amount)
      //const msg = {rewards:{retrieve:{amount: String(amount)}}};
      //this.contract.handle(msg)
      //super.retrieve(amount)
    //} catch (e) {
      ////console.error(e)
    //}
  //}
  //claim () {
    //this.contract.sender = this.address
    //try {
      //const result = this.contract.handle({ rewards: { claim: {} } })
      //const reward = Number(result.log.reward)
      //return this.doClaim(reward)
    //} catch (e) {
      //console.error(e)
      //return 0
    //}
  //}
//}

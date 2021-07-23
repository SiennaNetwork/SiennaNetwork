import { encode, decode } from './helpers'
import { UIContext } from './widgets'
import { User, Pool } from './contract_base'
import initRewards, * as Bound from '../target/web/rewards.js'

// wrapper classes on the js side too... -----------------------------------------------------------
class Rewards {
  index = 0
  contract = new Bound.Contract()
  debug = false
  init (msg: object) {
    this.index += 1
    if (this.debug) console.debug(`init> ${this.index}`, msg)
    const res = decode(this.contract.init(encode(msg)))
    if (this.debug) console.debug(`<init ${this.index}`, res)
    return res
  }
  query (msg: object) {
    this.index += 1
    if (this.debug) console.debug(`query> ${this.index}`, msg)
    const res = decode(this.contract.query(encode(msg)))
    if (this.debug) console.debug(`<query ${this.index}`, res)
    return res
  }
  handle (msg: object) {
    this.index += 1
    if (this.debug) console.debug(`handle> ${this.index}`, msg)
    const res = decode(this.contract.handle(encode(msg)))
    if (this.debug) console.debug(`<handle ${this.index}`, res)
    return res
  }
}

  //rewards.next_query_response = encode(JSON.stringify({
    //balance: { amount: "0" }
  //}))

  //console.log(decode(rewards.query(new QueryMsg({
    //user_info: { at: 0, address: "", key: "" }
  //})).json))

// wasm module load & init -------------------------------------------------------------------------
export default async function initReal () {
  // thankfully wasm-pack/wasm-bindgen left an escape hatch
  // because idk wtf is going on with the default loading code
  const url = new URL('rewards_bg.wasm', location.href)
      , res = await fetch(url.toString())
      , buf = await res.arrayBuffer()
  await initRewards(buf)
}

// pool api ----------------------------------------------------------------------------------------
export class RealPool extends Pool {
  contract: Rewards = new Rewards()
  constructor (ui: UIContext) {
    super(ui)
    this.contract.init({
      reward_token: { address: "", code_hash: "" },
      lp_token:     { address: "", code_hash: "" },
      viewing_key:  ""
    })
  }
  get_info () {
    return this.contract.query({ pool_info: { at: 0 } })
  }
  update () {
    const info = this.get_info()
    this.balance     = 0 // TODO in contract
    this.last_update = info.pool_last_update
    this.lifetime    = info.pool_lifetime
    this.locked      = info.pool_locked
  }
}

// user api ----------------------------------------------------------------------------------------
export class RealUser extends User {

  address: string

  get contract () {
    return (this.pool as RealPool).contract
  }

  constructor (ui: UIContext, pool: Pool, name: string, balance: number) {
    super(ui, pool, name, balance)
    this.address = this.name
    this.contract.contract.sender = encode(this.address)
    this.contract.handle({ set_viewing_key: { key: "" } })
  }

  update () {
    this.contract.contract.next_query_response = encode({balance:{amount:String(this.balance)}})
    const info = this.contract.query({user_info: { at: 0, address: this.address, key: "" }}).user_info
    this.last_update = info.user_last_update
    this.lifetime    = Number(info.user_lifetime)
    this.locked      = Number(info.user_locked)
    this.age         = info.user_age
    this.cooldown    = 0
    this.earned      = Number(info.user_earned)
    this.claimed     = Number(info.user_claimed)
    this.claimable   = Number(info.user_claimable)
    super.update()
  }

  lock (amount: number) {
    this.contract.contract.sender = encode(this.address)
    try {
      console.debug('lock', amount, this.contract.handle({ lock: { amount: String(amount) } }))
      super.lock(amount)
    } catch (e) {
      console.error(e)
    }
  }

  retrieve (amount: number) {
    this.contract.contract.sender = encode(this.address)
    try {
      console.debug('retrieve', amount, this.contract.handle({ retrieve: { amount: String(amount) } }))
      super.retrieve(amount)
    } catch (e) {
      console.error(e)
    }
  }

  claim () {
    this.contract.contract.sender = encode(this.address)
    try {
      console.debug('claim', this.contract.handle({ claim: {} }))
      const reward = super.claim()
      return reward
    } catch (e) {
      console.error(e)
      return 0
    }
  }
}

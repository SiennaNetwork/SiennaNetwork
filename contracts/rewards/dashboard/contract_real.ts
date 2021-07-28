import { encode, decode } from './helpers'
import { UIContext } from './widgets'
import { T, User, Pool, THRESHOLD, COOLDOWN } from './contract_base'
import initRewards, * as Bound from '../target/web/rewards.js'

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
    //if (this.debug) console.debug(`init> ${this.index}`, msg)
    const res = decode(this.contract.init(encode(msg)))
    //if (this.debug) console.debug(`<init ${this.index}`, res)
    return res
  }
  query (msg: object) {
    this.index += 1
    this.block = T.T
    //if (this.debug) console.debug(`query> ${this.index}`, msg)
    const res = decode(this.contract.query(encode(msg)))
    //if (this.debug) console.debug(`<query ${this.index}`, res)
    return res
  }
  handle (msg: object) {
    this.index += 1
    this.block = T.T
    //if (this.debug) console.debug(`handle> ${this.index}`, msg)
    const res: HandleResponse = decode(this.contract.handle(encode(msg)))
    res.log = Object.fromEntries(Object
      .values(res.log as object)
      .map(({key, value})=>[key, value]))
    if (Object.keys(res.log).length > 0) console.log(res.log)
    //if (this.debug) console.debug(`<handle ${this.index}`, res)
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
}

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
      viewing_key:  "",
      threshold:    THRESHOLD,
      cooldown:     COOLDOWN
    })
    this.ui.log.close.onclick = this.close.bind(this)
  }
  update () {
    this.contract.next_query_response = {balance:{amount:String(this.balance)}}
    const info = this.contract.query({pool_info:{at:T.T}}).pool_info
    //console.log(info)
    this.last_update = info.pool_last_update
    this.lifetime    = info.pool_lifetime
    this.locked      = info.pool_locked
    this.claimed     = info.pool_claimed
    this.threshold   = info.pool_threshold
    this.cooldown    = info.pool_cooldown
    this.liquid      = info.pool_liquid
    super.update()
  }
  close () {
    this.contract.sender = ""
    this.contract.handle({close_pool:{message:"pool closed"}})
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
    this.contract.sender = this.address
    this.contract.handle({ set_viewing_key: { key: "" } })
  }

  update () {
    // mock the user's balance - actually stored on this same object
    // because we don't have a snip20 contract to maintain it
    this.contract.next_query_response = {balance:{amount:String(this.pool.balance)}}

    // get the user's info as stored and calculated by the rewards contract
    // presuming the above mock balance
    const info = this.contract.query({user_info: { at: T.T, address: this.address, key: "" }}).user_info
    this.last_update = info.user_last_update
    this.lifetime    = Number(info.user_lifetime)
    this.locked      = Number(info.user_locked)
    this.share       = Number(info.user_share)
    this.age         = Number(info.user_age)
    this.earned      = Number(info.user_earned)
    this.claimed     = Number(info.user_claimed)
    this.claimable   = Number(info.user_claimable)
    this.cooldown    = Number(info.user_cooldown)
    super.update()
  }

  lock (amount: number) {
    this.contract.sender = this.address
    try {
      //console.debug('lock', amount)
      this.contract.handle({ lock: { amount: String(amount) } })
      super.lock(amount)
    } catch (e) {
      //console.error(e)
    }
  }

  retrieve (amount: number) {
    this.contract.sender = this.address
    try {
      //console.debug('retrieve', amount)
      this.contract.handle({ retrieve: { amount: String(amount) } })
      super.retrieve(amount)
    } catch (e) {
      //console.error(e)
    }
  }

  claim () {
    this.contract.sender = this.address
    try {
      const result = this.contract.handle({ claim: {} })
      const reward = Number(result.log.reward)
      return this.doClaim(reward)
    } catch (e) {
      console.error(e)
      return 0
    }
  }
}

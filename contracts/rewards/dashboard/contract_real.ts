import { COLORS } from './helpers'
import { UIContext } from './widgets'
import initRewards, {
  Contract      as NativeRewards,
  InitMsg       as NativeInitMsg,
  QueryMsg      as NativeQueryMsg,
  QueryResponse as NativeQueryResponse,
  HandleMsg     as NativeHandleMsg,
  Env,
} from '../target/web/rewards.js'

// settings ----------------------------------------------------------------------------------------
const TIME_SCALE          = 30
const FUND_PORTIONS       = 120
const FUND_PORTION        = 2500
const FUND_INTERVAL       = 17280/TIME_SCALE
const COOLDOWN            = FUND_INTERVAL
const THRESHOLD           = FUND_INTERVAL
const USER_GIVES_UP_AFTER = Infinity
const MAX_USERS           = 100
const MAX_INITIAL         = 1000

// convert from string to Utf8Array ----------------------------------------------------------------
const enc = new TextEncoder()
const encode = (x: any) => enc.encode(JSON.stringify(x))
const dec = new TextDecoder()
const decode = (x: any) => JSON.parse(dec.decode(x))
class Rewards extends NativeRewards {
}
class InitMsg extends NativeInitMsg {
  constructor (msg: object) {
    super(enc.encode(JSON.stringify(msg)))
  }
}
class QueryMsg extends NativeQueryMsg {
  constructor (msg: object) {
    super(enc.encode(JSON.stringify(msg)))
  }
}
class QueryResponse extends NativeQueryResponse {
  constructor (msg: object) {
    super(enc.encode(JSON.stringify(msg)))
  }
}
class HandleMsg extends NativeHandleMsg {
  constructor (msg: object) {
    super(enc.encode(JSON.stringify(msg)))
  }
}

export default async function initReal (ui: UIContext) {

  const T = { T: 0 }

  // wasm module load & init -----------------------------------------------------------------------
  // thankfully wasm-pack/wasm-bindgen left an escape hatch
  // because idk wtf is going on with the default loading code
  const url = new URL('rewards_bg.wasm', location.href)
      , res = await fetch(url.toString())
      , buf = await res.arrayBuffer()
  await initRewards(buf/*, {
    custom_query (arg) {
      console.log("custom_query", arg)
      return arg
    }
  }*/)

  // instantiate a reward pool ---------------------------------------------------------------------
  let env
  const rewards = new Rewards()

  env = new Env(BigInt(0))

  console.log(decode(rewards.init(env, new InitMsg({
    reward_token: { address: "", code_hash: "" },
    lp_token:     { address: "", code_hash: "" },
    viewing_key:  ""
  })).json))

  console.log(decode(rewards.query(new QueryMsg({
    pool_info: { at: 0 }
  })).json))

  env = new Env(BigInt(1))

  console.log(decode(rewards.handle(env, new HandleMsg({
    set_viewing_key: { key: "" }
  })).json))

  rewards.next_query_response = enc.encode(JSON.stringify({
    balance: { amount: "0" }
  }))

  console.log(decode(rewards.query(new QueryMsg({
    user_info: { at: 0, address: "", key: "" }
  })).json))

  class RPT {
    interval  = FUND_INTERVAL
    portion   = FUND_PORTION
    remaining = FUND_PORTIONS
  }

  class Pool {
    ui: UIContext
    get info () {
      return decode(rewards.query(
        new QueryMsg({ pool_info: { at: T.T } })
      ).json)
    }
    get balance () { return 0 /* TODO in contract */ }
    get last_update () { return this.info.pool_last_update }
    get lifetime () { return this.info.pool_lifetime }
    get locked () { return this.info.pool_locked }
  }

  class User {
    ui: UIContext
    pool: Pool
    name: string
    balance: number

    get info () {
      return decode(rewards.query(
        new QueryMsg({user_info: { at: T.T, address: "", key: "" }})
      ).json)
    }
    get last_update () { return this.info.user_last_update }
    get lifetime () { return this.info.user_lifetime }
    get locked () { return this.info.user_locked }
    get age () { return this.info.user_age }
    get cooldown () { return 0 }
    get earned () { return this.info.user_earned }
    get claimed () { return this.info.user_claimed }
    get claimable () { return this.info.user_claimable }

    constructor (ui: UIContext, pool: Pool, name: string, balance: number) {
      this.ui      = ui
      this.pool    = pool
      this.name    = name
      this.balance = balance
    }

    colors () {
      return COLORS(this.pool, this)
    }

    lock (amount: number) {}

    retrieve (amount: number) {}

    claim () {}

    update () {}
  }

  return {
    Pool,
    pool: {
      update () {
        const TODO = 'TODO'
        ui.log.now.textContent = `block ${TODO}`
        ui.log.balance.textContent = `reward budget: ${TODO}`
        ui.log.remaining.textContent = `${TODO} days remaining`
      } 
    },
    users: {}
  }
}

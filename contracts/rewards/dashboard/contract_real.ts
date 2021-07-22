import { UIContext } from './widgets'
import { User, Pool } from './contract_base'
import initRewards, {
  Contract      as NativeRewards,
  InitMsg       as NativeInitMsg,
  QueryMsg      as NativeQueryMsg,
  QueryResponse as NativeQueryResponse,
  HandleMsg     as NativeHandleMsg,
  Env,
} from '../target/web/rewards.js'

// convert from string to Utf8Array ----------------------------------------------------------------
const enc = new TextEncoder()
const encode = (x: any) => enc.encode(JSON.stringify(x))
const dec = new TextDecoder()
const decode = (x: any) => JSON.parse(dec.decode(x))

// wrapper classes on the js side too... -----------------------------------------------------------
class Rewards extends NativeRewards {
}
class InitMsg extends NativeInitMsg {
  constructor (msg: object) {
    super(encode(JSON.stringify(msg)))
  }
}
class QueryMsg extends NativeQueryMsg {
  constructor (msg: object) {
    super(encode(JSON.stringify(msg)))
  }
}
class HandleMsg extends NativeHandleMsg {
  constructor (msg: object) {
    super(encode(JSON.stringify(msg)))
  }
}

export default async function initReal () {
  // wasm module load & init -----------------------------------------------------------------------
  // thankfully wasm-pack/wasm-bindgen left an escape hatch
  // because idk wtf is going on with the default loading code
  const url = new URL('rewards_bg.wasm', location.href)
      , res = await fetch(url.toString())
      , buf = await res.arrayBuffer()
  await initRewards(buf)
}

export class RealPool extends Pool {
  contract: Rewards = new Rewards()
  env:      Env     = new Env(BigInt(0))
  constructor (ui: UIContext) {
    super(ui)
    console.debug('init rewards', decode(this.contract.init(this.env, new InitMsg({
      reward_token: { address: "", code_hash: "" },
      lp_token:     { address: "", code_hash: "" },
      viewing_key:  ""
    })).json))
  }
  get info () {
    return decode(this.contract.query(
      new QueryMsg({ pool_info: { at: 0 } })
    ).json)
  }
  update () {
    this.balance     = 0 // TODO in contract
    this.last_update = this.info.pool_last_update
    this.lifetime    = this.info.pool_lifetime
    this.locked      = this.info.pool_locked
  }
}

//rewards.next_query_response = encode(JSON.stringify({
  //balance: { amount: "0" }
//}))

//console.log(decode(rewards.query(new QueryMsg({
  //user_info: { at: 0, address: "", key: "" }
//})).json))

export class RealUser extends User {
  constructor (ui: UIContext, pool: Pool, name: string, balance: number) {
    super(ui, pool, name, balance)
    const { contract, env } = this.pool as RealPool
    console.debug('set user vk', decode(contract.handle(env,
      new HandleMsg({ set_viewing_key: { key: "" } })
    ).json))
  }

  update () {
    const { contract } = this.pool as RealPool
    const msg = new QueryMsg({user_info: { at: 0, address: "", key: "" }})
    const info = decode(contract.query(msg))
    this.last_update = info.user_last_update
    this.lifetime    = info.user_lifetime
    this.locked      = info.user_locked
    this.age         = info.user_age
    this.cooldown    = 0
    this.earned      = info.user_earned
    this.claimed     = info.user_claimed
    this.claimable   = info.user_claimable
  }

  lock (amount: number) {
    super.lock(amount)
  }

  retrieve (amount: number) {
    if (this.locked < amount) return
    super.retrieve(amount)
  }

  claim () {
    const reward = super.claim()
    return reward
  }
}

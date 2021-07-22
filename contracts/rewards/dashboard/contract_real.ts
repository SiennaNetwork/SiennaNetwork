import { encode, decode } from './helpers'
import { UIContext } from './widgets'
import { User, Pool } from './contract_base'
import initRewards, {
  Contract as NativeRewards,
  InitMsg,
  QueryMsg,
  //QueryResponse,
  HandleMsg,
  Env,
} from '../target/web/rewards.js'

// wrapper classes on the js side too... -----------------------------------------------------------
class Rewards extends NativeRewards {
  env: Env = new Env(BigInt(0))
  init (msg: object) {
    return decode(super.init(this.env, new InitMsg(encode(JSON.stringify(msg)))).json)
  }
  query (msg: object) {
    return decode(super.query(new QueryMsg(encode(JSON.stringify(msg)))).json)
  }
  handle (msg: object) {
    return decode(super.handle(this.env, new HandleMsg(encode(JSON.stringify(msg)))).json)
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
  constructor (ui: UIContext) {
    super(ui)
    console.debug('init rewards', this.contract.init({
      reward_token: { address: "", code_hash: "" },
      lp_token:     { address: "", code_hash: "" },
      viewing_key:  ""
    }))
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

//rewards.next_query_response = encode(JSON.stringify({
  //balance: { amount: "0" }
//}))

//console.log(decode(rewards.query(new QueryMsg({
  //user_info: { at: 0, address: "", key: "" }
//})).json))

export class RealUser extends User {

  get contract () {
    return (this.pool as RealPool).contract
  }

  constructor (ui: UIContext, pool: Pool, name: string, balance: number) {
    super(ui, pool, name, balance)
    console.debug('set user vk', this.contract.handle({ set_viewing_key: { key: "" } }))
  }

  update () {
    const info = this.contract.query({user_info: { at: 0, address: "", key: "" }})
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
    console.debug('lock', this.contract.handle({ lock: { amount: String(amount) } }))
    super.lock(amount)
  }

  retrieve (amount: number) {
    console.debug('retrieve', this.contract.handle({ retrieve: { amount: String(amount) } }))
    super.retrieve(amount)
  }

  claim () {
    const reward = super.claim()
    console.debug('claim', this.contract.handle({ claim: {} }))
    return reward
  }
}

import { UIContext } from './contract_mock'
import initRewards, {
  Contract as Rewards,
  Env,
  Init, Q
} from '../target/web/rewards.js'

// converts from string to Utf8Array ---------------------------------------------------------------
const enc = new TextEncoder()

export default async function initReal (ui: UIContext) {

  // wasm module load & init -----------------------------------------------------------------------
  // thankfully wasm-pack/wasm-bindgen left an escape hatch
  // because idk wtf is going on with the default loading code
  const url = new URL('rewards_bg.wasm', location.href)
      , res = await fetch(url.toString())
      , buf = await res.arrayBuffer()
  await initRewards(buf)

  // instantiate a reward pool ---------------------------------------------------------------------
  const rewards = new Rewards()
  const init = enc.encode(`{
    "reward_token": { "address": "", "code_hash": "" },
    "viewing_key": ""
  }`)
  const init_result = rewards.init(new Env(BigInt(0)), new Init(init))
  console.log({rewards, init_result})

  const q_pool = rewards.query(new Q(enc.encode(`{
    "pool_info": { "at": "0" }
  }`)))

  const q_user = rewards.query(new Q(enc.encode(`{
    "user_info": { "at": "0", "address": "", "vk": "" }
  }`)))

  return {
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

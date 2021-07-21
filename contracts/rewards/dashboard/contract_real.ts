import initRewards, { Contract as Rewards, Env, Init } from '../target/web/rewards.js'

;(async () => {
  const url      = new URL('rewards_bg.wasm', location.href)
      , res      = await fetch(url.toString())
      , buf      = await res.arrayBuffer()
      , enc      = new TextEncoder()

  await initRewards(buf)
  const rewards = new Rewards()
  const init = enc.encode(`{
    "reward_token": {
      "address": "",
      "code_hash": ""
    },
    "viewing_key": ""
  }`)
  const init_result = rewards.init(new Env(), new Init(init))
  console.log({rewards, init_result})
})()

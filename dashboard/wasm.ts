import initSIENNA,  * as SIENNA  from './artifacts/sienna/sienna.js'
import initLPToken, * as LPToken from './artifacts/lptoken/lptoken.js'
import initMGMT,    * as MGMT    from './artifacts/mgmt/mgmt.js'
import initRPT,     * as RPT     from './artifacts/rpt/rpt.js'
import initRewards, * as Rewards from './artifacts/rewards/rewards.js'

const CONTRACTS: Array<[string, Function]> = [
  ['sienna/sienna_bg.wasm',   initSIENNA],
  ['lptoken/lptoken_bg.wasm', initLPToken],
  ['mgmt/mgmt_bg.wasm',       initMGMT],
  ['rpt/rpt_bg.wasm',         initRPT],
  ['rewards/rewards_bg.wasm', initRewards]
]

export default initContracts()

async function initContracts () {
  for (const [blob, init] of CONTRACTS) {
    console.debug(`init`, blob)
    const url = new URL(blob, location.href)
        , res = await fetch(url.toString())
        , buf = await res.arrayBuffer()
    await init(buf)
  }
  return {
    SIENNA:  SIENNA.Contract,
    LPToken: LPToken.Contract,
    MGMT:    MGMT.Contract,
    RPT:     RPT.Contract,
    Rewards: Rewards.Contract
  }
}

import { encode, decode } from '../helpers'
import Component from '../Component'

import initSIENNA,  * as SIENNA  from '../artifacts/sienna/sienna.js'
import initLPToken, * as LPToken from '../artifacts/lptoken/lptoken.js'
import initMGMT,    * as MGMT    from '../artifacts/mgmt/mgmt.js'
import initRPT,     * as RPT     from '../artifacts/rpt/rpt.js'
import initRewards, * as Rewards from '../artifacts/rewards/rewards.js'

const CONTRACTS: Array<[string, Function]> = [
  ['sienna/sienna_bg.wasm',   initSIENNA],
  ['lptoken/lptoken_bg.wasm', initLPToken],
  ['mgmt/mgmt_bg.wasm',       initMGMT],
  ['rpt/rpt_bg.wasm',         initRPT],
  ['rewards/rewards_bg.wasm', initRewards]
]

export async function initContracts () {

  await Promise.all(CONTRACTS.map(async ([blob,init])=>{
    console.debug(`init`, blob)
    const url = new URL(blob, location.href)
        , res = await fetch(url.toString())
        , buf = await res.arrayBuffer()
    await init(buf)
  }))

  return {
    SIENNA:  SIENNA.Contract,
    LPToken: LPToken.Contract,
    MGMT:    MGMT.Contract,
    RPT:     RPT.Contract,
    Rewards: Rewards.Contract
  }

}

export class Querier {
  contracts: Record<string, ContractComponent> = {}
  add (addr: string, comp: ContractComponent) {
    this.contracts[addr] = comp
  }
  query (request: any) {
    console.debug('querier', request)
    const {contract_addr, msg} = request.wasm.smart
    const target = this.contracts[contract_addr]
    if (target) {
      return target.query(msg)
    } else {
      throw new Error(`can't query unknown address ${contract_addr}`)
    }
  }
}

export const querier = new Querier()

export default abstract class ContractComponent extends Component {

  #contract: any

  setup (Contract: any) {
    this.#contract = new Contract("", "")
    this.#contract.init(encode(this.initMsg))
    this.#contract.querier_callback = (data: string) => {
      try {
        const request = JSON.parse(data)
        const msg = request.wasm.smart.msg
        request.wasm.smart.msg = JSON.parse(atob(msg).trim())
        return JSON.stringify(querier.query(request))
      } catch (e) {
        console.error(e)
        throw e
      }
    }
  }

  abstract readonly initMsg: any

  abstract update (): void

  query (msg: any) {
    console.debug('query', this.constructor.name, msg)
    return decode(this.#contract.query(encode(msg)))
  }

  handle (sender: any, msg: any) {
    console.debug('handle', sender, msg)
    this.#contract.sender = encode(sender)
    let {messages, log, data} = decode(this.#contract.handle(encode(msg)))
    data = JSON.parse(atob(data))
    return {messages, log, data}
  }

}

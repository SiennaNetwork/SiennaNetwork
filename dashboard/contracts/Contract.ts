import { encode, decode } from '../helpers'
import Component from '../Component'

import initSIENNA,  * as SIENNA  from '../artifacts/sienna/sienna.js'
import initLPToken, * as LPToken from '../artifacts/lptoken/lptoken.js'
import initMGMT,    * as MGMT    from '../artifacts/mgmt/mgmt.js'
import initRPT,     * as RPT     from '../artifacts/rpt/rpt.js'
import initRewards, * as Rewards from '../artifacts/rewards/rewards.js'

export interface IContract {
  query (msg: any): any
}

export class Cosmos {

  static default = new Cosmos()

  static CONTRACTS: Array<[string, Function]> = [
    ['sienna/sienna_bg.wasm',   initSIENNA],
    ['lptoken/lptoken_bg.wasm', initLPToken],
    ['mgmt/mgmt_bg.wasm',       initMGMT],
    ['rpt/rpt_bg.wasm',         initRPT],
    ['rewards/rewards_bg.wasm', initRewards]
  ]

  static async loadContracts () {

    await Promise.all(Cosmos.CONTRACTS.map(async ([blob, load])=>{
      const url = new URL(blob, location.href)
      console.debug({load:url.toString()})
      const res = await fetch(url.toString())
      const buf = await res.arrayBuffer()
      await load(buf)
    }))

    return {
      SIENNA:  SIENNA.Contract,
      LPToken: LPToken.Contract,
      MGMT:    MGMT.Contract,
      RPT:     RPT.Contract,
      Rewards: Rewards.Contract
    }

  }

  contracts: Record<string, IContract> = {}

  add (addr: string, comp: IContract) {
    this.contracts[addr] = comp
  }

  query (request: any) {
    const {contract_addr, msg} = request.wasm.smart
    const target = this.contracts[contract_addr]
    console.debug('cosmos', request, target)
    if (target) {
      console.debug('queryresponse', target.query(msg))
      return target.query(msg)
    } else {
      console.error(`can't query unknown address ${contract_addr}`)
      return {}
    }
  }

  queryCallback (data: string) {
    try {
      const request = JSON.parse(data)
      const msg = request.wasm.smart.msg
      request.wasm.smart.msg = JSON.parse(atob(msg).trim())
      return JSON.stringify(Cosmos.default.query(request))
    } catch (e) {
      console.error(e)
      return JSON.stringify({})
    }
  }

  Contract = class CosmosContractComponent extends Component {

    #wasm: any

    set sender (addr: string) { this.#wasm.sender = encode(addr) }

    setup (WASM: any, addr: string, hash: string) {
      this.#wasm = new WASM(addr, hash)
      this.#wasm.querier_callback = Cosmos.default.queryCallback
      this.sender = "Admin"
      this.init(this.initMsg)
      this.update()
    }

    update () {
      console.warn('empty update method called')
    }

    initMsg: any = {}

    init (msg: any): any {
      return decode(this.#wasm.init(encode(msg)))
    }

    query (msg: any) {
      console.debug('query', this.constructor.name, msg)
      return decode(this.#wasm.query(encode(msg)))
    }

    handle (sender: string, msg: any) {
      console.debug('handle', sender, msg)
      this.sender = sender
      let {messages, log, data} = decode(this.#wasm.handle(encode(msg)))
      if (data) data = JSON.parse(atob(data))
      return {messages, log, data}
    }

  }

}

export default Cosmos.default.Contract

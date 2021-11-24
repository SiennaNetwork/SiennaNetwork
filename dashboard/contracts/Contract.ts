import { encode, decode } from '../helpers'
import Component from '../Component'

import initSIENNA,  * as SIENNA  from '../artifacts/sienna/sienna.js'
import initLPToken, * as LPToken from '../artifacts/lptoken/lptoken.js'
import initMGMT,    * as MGMT    from '../artifacts/mgmt/mgmt.js'
import initRPT,     * as RPT     from '../artifacts/rpt/rpt.js'
import initRewards, * as Rewards from '../artifacts/rewards/rewards.js'

const debug = (obj:any) => console.debug(JSON.stringify(obj))

export interface IContract {
  addr: string
  query (msg: any): any
  handle (sender: string, msg: any): any
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
      debug({load:url.toString()})
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
    debug({inter_contract_query:{target:target.addr, request}})
    if (target) {
      const response = target.query(msg)
      debug({inter_contract_query_response:response})
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
      debug({inter_contract_query_callback:request})
      return JSON.stringify(Cosmos.default.query(request))
    } catch (e) {
      console.error(e)
      return JSON.stringify({})
    }
  }

  processMessages (sender: string, messages: Array<any>) {
    for (const message of messages) {
      const addr = message.wasm.execute.contract_addr
      const msg  = JSON.parse(atob(message.wasm.execute.msg))
      this.contracts[addr].handle(sender, msg)
      debug({process:message})
    }
  }

  Contract = class CosmosContractComponent extends Component {

    #wasm: any

    set sender (addr: string) { this.#wasm.sender = encode(addr) }

    addr: string = ""
    hash: string = ""

    setup (WASM: any, addr: string, hash: string) {
      this.addr = addr
      this.hash = hash
      this.#wasm = new WASM(encode(addr), encode(hash))
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
      debug({init:{sender:this.sender, msg}})
      const response = decode(this.#wasm.init(encode(msg)))
      debug({init_response:response})
      Cosmos.default.processMessages(this.addr, response.messages)
      return response
    }

    query (msg: any) {
      debug({query:{contract:this.constructor.name, msg}})
      return decode(this.#wasm.query(encode(msg)))
    }

    handle (sender: string, msg: any) {
      debug({handle:{sender, msg}})
      this.sender = sender
      const response = decode(this.#wasm.handle(encode(msg)))
      if (response.data) response.data = JSON.parse(atob(response.data))
      debug({handle_response:response})
      Cosmos.default.processMessages(this.addr, response.messages)
      return response
    }

  }

}

export default Cosmos.default.Contract

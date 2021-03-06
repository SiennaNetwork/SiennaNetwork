import { h, encode, decode } from './helpers'
import Component from './Component'

import initSIENNA,  * as SIENNA  from './artifacts/sienna/sienna.js'
import initLPToken, * as LPToken from './artifacts/lptoken/lptoken.js'
import initMGMT,    * as MGMT    from './artifacts/mgmt/mgmt.js'
import initRPT,     * as RPT     from './artifacts/rpt/rpt.js'
import initRewards, * as Rewards from './artifacts/rewards/rewards.js'

import Field  from './widgets/Field'
import Button from './widgets/Button'

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

  time = 0

  queryCallback (data: string) {
    let response = {}
    try {
      const request = JSON.parse(data)
      const {contract_addr, msg} = request.wasm.smart
      request.wasm.smart.msg = JSON.parse(atob(msg).trim())
      const target = Cosmos.default.contracts[contract_addr]
      debug({inter_contract_query:{target:target.addr, request}})
      if (target) {
        response = target.query(request.wasm.smart.msg)
        debug({inter_contract_query_response:response})
      } else {
        console.error(`can't query unknown address ${contract_addr}`)
      }
    } catch (e) {
      console.error(e)
    }
    return btoa(JSON.stringify(response))
  }

  passMessages (sender: string, messages: Array<any>) {
    for (const message of messages) {
      const addr = message.wasm.execute.contract_addr
      const msg  = JSON.parse(atob(message.wasm.execute.msg))
      debug({pass:{addr, sender, msg}})
      this.contracts[addr].handle(sender, msg)
    }
  }

  Contract = class CosmosContractComponent extends Component {

    #wasm: any

    #sender: any
    get sender () {
      return this.#sender
    }
    set sender (addr: string) {
      this.#sender = addr
      this.#wasm.sender = encode(addr)
    }

    addr: string = ""
    hash: string = ""

    setup (WASM: any, addr: string, hash: string) {
      this.addr = addr
      this.hash = hash
      this.#wasm = new WASM(addr, hash);
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
      this.#wasm.time = BigInt(Cosmos.default.time)
      console.debug(`${this.addr} was initialized by ${this.sender} with ${JSON.stringify(msg, null, 2)}`)
      //debug({addr:this.addr,init:{sender:this.sender, msg}})
      const response = decode(this.#wasm.init(encode(msg)))
      console.debug(`${this.addr} init ok: ${JSON.stringify(response, null, 2)}`)
      //debug({addr:this.addr,init_response:response})
      Cosmos.default.passMessages(this.addr, response.messages)
      return response
    }

    query (msg: any) {
      console.debug(`${this.addr} was queried: ${JSON.stringify(msg, null, 2)}`)
      //debug({addr:this.addr,query:{contract:this.constructor.name,addr:this.addr,msg}})
      const response = decode(this.#wasm.query(encode(msg)))
      console.debug(`${this.addr} responded: ${JSON.stringify(response, null, 2)}`)
      //debug({addr:this.addr,query_response:response})
      return response
    }

    handle (sender: string, msg: any) {
      console.log(this.#wasm.get_time)
      this.#wasm.time = BigInt(Cosmos.default.time)
      console.debug(`${this.addr} handled transaction by ${sender}: ${JSON.stringify(msg, null, 2)}`)
      //debug({addr:this.addr,handle:{sender, msg}})
      this.sender = sender
      const response = decode(this.#wasm.handle(encode(msg)))
      if (response.data) response.data = JSON.parse(atob(response.data))
      console.debug(`${this.addr} transaction ok: ${JSON.stringify(response, null, 2)}`)
      //debug({addr:this.addr,handle_response:response})
      Cosmos.default.passMessages(this.addr, response.messages)
      return response
    }

  }

}

export default Cosmos.default.Contract

type Timer = ReturnType<typeof setTimeout>

export class Environment extends Component {

  static TAG   = 'x-environment'
  static CLASS = 'Outside Environment'
  static make  = (dashboard: any) =>
    h(this.TAG, { className: this.CLASS, dashboard })
  static _ = customElements.define(this.TAG, this)

  #dashboard: any = null
  get dashboard () { return this.#dashboard }
  set dashboard (v: any) { this.#dashboard = v }

  time = 0
  rate = [600, 33]
  timer: Timer|null = null

  start () {
    this.timer = setInterval(this.update.bind(this), this.rate[1])
  }

  pause () {
    if (this.timer) clearInterval(this.timer)
    this.timer = null
  }

  update () {
    this.time += this.rate[0]
    Cosmos.default.time = this.time
    this.ui.time.value = `${this.time}s`
    if (this.time % 86400 === 0) this.dashboard.microservice.nextEpoch()
    this.dashboard.update()
  }

  ui = {
    //title: this.add(h('header', { textContent: 'Environment' })),
    time:  this.add(Field('Time', `${this.time}s`)),
    rate:  this.add(Field('Speed', `${this.rate[0]}s per ${this.rate[1]}ms`)),
    start: this.add(Button.make('START', () => this.start())),
    pause: this.add(Button.make('PAUSE', () => this.pause())),
  }

}

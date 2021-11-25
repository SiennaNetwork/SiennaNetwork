import { h, append, throttle } from './helpers'
import Component       from './Component'
import Button          from './widgets/Button'
import SNIP20          from './contracts/SNIP20'
import Rewards         from './contracts/Rewards'
import { Environment } from './Cosmos'
import { MGMT, RPT, Microservice } from './contracts/TGE'

import {Cosmos} from './Cosmos'

type Contracts = Record<string, any> 

export default class Dashboard extends Component {

  static TAG   = 'x-dashboard'
  static CLASS = 'Outside Dashboard'
  static make  = (contracts: Record<string, any>) =>
    h(this.TAG, { className: this.CLASS, contracts })
  static _ = customElements.define(this.TAG, this)

  row1 = this.add(h('div', { className: 'Row', style: 'flex-grow:0;flex-shrink:0' }))
  environment  = append(this.row1)(Environment.make(this))
  microservice = append(this.row1)(Microservice.make(this))

  row2 = this.add(h('div', { className: 'Row', style: 'flex-grow:0' }))
  lpToken = append(this.row2)(SNIP20.make('LPTOKEN'))
  sienna  = append(this.row2)(SNIP20.make('SIENNA'))
  mgmt    = append(this.row2)(MGMT.make())
  rpt     = append(this.row2)(RPT.make(this))

  row3 = this.add(h('div', { className: 'Row', style: 'flex-grow:3' }))
  rewards_v3 = append(this.row3)(Rewards.make(this, 'v3'))
  migrate    = append(this.row3)(Button('Migrate', () => this.performMigration()))
  rewards_v4 = append(this.row3)(Rewards.make(this, 'v4'))

  #contracts: Contracts|null = null
  set contracts (v: Contracts) {
    if (this.#contracts === null) {
      this.#contracts = v
      this.setup()
    } else {
      throw new Error('contracts already provided')
      // TODO: hot code reloading (export storage from fadroma-bind-js)
    }
  }
  get contracts () {
    if (this.#contracts === null) {
      throw new Error('contracts not provided')
    } else {
      return this.#contracts
    }
  }

  setup () {
    console.log(this)
    this.sienna.setup(this.contracts.SIENNA, "SIENNA_addr", "SIENNA_hash")
    Cosmos.default.add('SIENNA_addr', this.sienna)

    this.mgmt.setup(this.contracts.MGMT, "MGMT_addr", "MGMT_hash")
    Cosmos.default.add('MGMT_addr', this.mgmt)

    this.rpt.setup(this.contracts.RPT, "SPLIT_RPT", "RPT_hash")
    Cosmos.default.add('SPLIT_RPT', this.rpt)

    this.lpToken.setup(this.contracts.LPToken, "LPTOKEN_addr", "LPTOKEN_hash")
    Cosmos.default.add('LPTOKEN_addr', this.lpToken)

    this.rewards_v3.setup(this.contracts.Rewards, "REWARDS_V3_addr", "REWARDS_V3_hash")
    Cosmos.default.add('REWARDS_V3_addr', this.rewards_v3)

    this.rewards_v4.setup(this.contracts.Rewards, "REWARDS_V4_addr", "REWARDS_V4_hash")
    Cosmos.default.add('REWARDS_V4_addr', this.rewards_v4)

    this.rewards_v3.resize()
    this.rewards_v4.resize()
    window.addEventListener('resize', throttle(100, () => {
      this.rewards_v3.resize()
      this.rewards_v4.resize()
    }))

    for (const contract of [this.sienna, this.lpToken]) {
      contract.register('Admin')
      contract.register('MGMT')
      contract.register('SPLIT_RPT')
      contract.register('REWARDS_V3_addr')
      contract.register('REWARDS_V4_addr')
    }

    this.sienna.handle("Admin", {
      set_minters:{minters:["MGMT_addr"]}
    })
    this.sienna.handle("Admin", {
      change_admin:{address:"MGMT_addr"}
    })
    this.mgmt.launch()
  }

  performMigration () {
    this.rpt.performMigration()
  }

  ids: Array<string> = []
  nextUser = 1
  addUser (balance: BigInt) {
    const id = `User ${this.nextUser}`
    this.ids.push(id)
    this.sienna.register(id)
    this.lpToken.register(id)
    console.debug('MINT', id, balance)
    this.lpToken.mint(id, balance)
    this.rewards_v3.register(id)
    this.rewards_v4.register(id)
    this.nextUser++;
    return id
  }

  update () {
    this.rewards_v3.update()
    this.rewards_v4.update()
  }
}

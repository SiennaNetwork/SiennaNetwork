import { h, append } from './helpers'

import Component from './Component'

import Button       from './widgets/Button'
import Environment  from './widgets/Environment'
import Microservice from './widgets/Microservice'

import SNIP20  from './contracts/SNIP20'
import MGMT    from './contracts/MGMT'
import RPT     from './contracts/RPT'
import rewards, { Rewards } from './contracts/Rewards'

import {querier} from './contracts/Contract'

export class Dashboard extends Component {

  ui: Record<string, any> = {
    row1: this.add(h('div', { className: 'Row' })),
    row2: this.add(h('div', { className: 'Row' })),
    row3: this.add(h('div', { className: 'Row' })),
    row4: this.add(h('div', { className: 'Row' })),
  }

  environment  = append(this.ui.row1)(Environment())
  microservice = append(this.ui.row1)(Microservice())
  mgmt         = append(this.ui.row2)(MGMT())
  rpt          = append(this.ui.row2)(RPT())
  sienna       = append(this.ui.row3)(SNIP20('SIENNA'))
  lpToken      = append(this.ui.row3)(SNIP20('LPTOKEN'))
  rewards_v3   = append(this.ui.row4)(rewards(this, 'v3'))
  migrate      = append(this.ui.row4)(Button('Migrate'))
  rewards_v4   = append(this.ui.row4)(rewards(this, 'v4'))

  #contracts: Contracts|null = null
  set contracts (v: Contracts) {
    if (this.#contracts === null) {
      this.#contracts = v
      this.setup()
    } else {
      throw new Error('contracts already provided')
      // this is where hot code reloading can take place
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
    this.sienna.setup(this.contracts.SIENNA)
    querier.add('SIENNA_addr', this.sienna)

    this.mgmt.setup(this.contracts.MGMT)
    querier.add('MGMT_addr', this.mgmt)

    this.rpt.setup(this.contracts.RPT)
    querier.add('RPT_addr', this.rpt)

    this.lpToken.setup(this.contracts.LPToken)
    querier.add('LPTOKEN_addr', this.lpToken)

    this.rewards_v3.setup(this.contracts.Rewards)
    querier.add('REWARDS_V3_addr', this.rewards_v3)

    this.rewards_v4.setup(this.contracts.Rewards)
    querier.add('REWARDS_V4_addr', this.rewards_v4)

    for (const contract of [this.sienna, this.lpToken]) {
      contract.register('Admin')
      contract.register('MGMT')
      contract.register('RPT')
      contract.register('Rewards V3')
      contract.register('Rewards V4')
    }

    this.sienna.mint('MGMT', this.mgmt.total)
  }

  nextUser = 1
  addUser (pool: Rewards, stake: BigInt) {
    const id = `User ${this.nextUser}`
    this.sienna.register(id)
    this.lpToken.register(id)
    this.lpToken.mint(id, stake)
    this.rewards_v3.ui.users.register(id)
    this.rewards_v4.ui.users.register(id)
    pool.deposit(id, stake)
    console.log({pool, stake})
    this.nextUser++
  }
}

type Contracts = Record<string, any> 

customElements.define('x-dashboard', Dashboard)
export default function dashboard (contracts: Record<string, any>) {
  return h('x-dashboard', { contracts, className: 'Outside Dashboard' })
}

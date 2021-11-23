import { h } from './helpers'

import Component from './Component'

import Button       from './widgets/Button'
import Environment  from './widgets/Environment'
import Microservice from './widgets/Microservice'

import SNIP20  from './contracts/SNIP20'
import MGMT    from './contracts/MGMT'
import RPT     from './contracts/RPT'
import rewards, { Rewards } from './contracts/Rewards'

export class Dashboard extends Component {

  ui: Record<string, any> = {
    environment:  this.add(Environment()),
    sienna:       this.add(SNIP20('SIENNA')),
    mgmt:         this.add(MGMT()),
    rpt:          this.add(RPT()),
    microservice: this.add(Microservice()),
    lpToken:      this.add(SNIP20('LPTOKEN')),
    rewards_v3:   this.add(rewards(this, 'v3')),
    migrate:      this.add(Button('Migrate')),
    rewards_v4:   this.add(rewards(this, 'v4'))
  }

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
    this.ui.sienna.setup(this.contracts.SIENNA)
    this.ui.mgmt.setup(this.contracts.MGMT)
    this.ui.rpt.setup(this.contracts.RPT)
    this.ui.lpToken.setup(this.contracts.LPToken)
    this.ui.rewards_v3.setup(this.contracts.Rewards)
    this.ui.rewards_v4.setup(this.contracts.Rewards)
    for (const contract of [this.ui.sienna, this.ui.lpToken]) {
      contract.register('Admin')
      contract.register('MGMT')
      contract.mint('MGMT', this.ui.mgmt.total)
      contract.register('RPT')
      contract.register('Rewards V3')
      contract.register('Rewards V4')
    }
  }

  nextUser = 1
  addUser (pool: Rewards, stake: BigInt) {
    const id = `User ${this.nextUser}`
    this.ui.sienna.register(id)
    this.ui.lpToken.register(id)
    this.ui.lpToken.mint(id, stake)
    this.ui.rewards_v3.ui.users.register(id)
    this.ui.rewards_v4.ui.users.register(id)
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

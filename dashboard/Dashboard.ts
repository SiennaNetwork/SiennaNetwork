import { h, append } from './helpers'

import Component from './Component'

import Environment  from './Environment'
import Microservice from './Microservice'

import SNIP20  from './SNIP20'
import MGMT    from './MGMT'
import RPT     from './RPT'
import Rewards from './Rewards'

export class Dashboard extends Component {

  ui: Record<string, any> = {
    environment:  this.add(Environment()),
    sienna:       this.add(SNIP20('SIENNA')),
    mgmt:         this.add(MGMT()),
    rpt:          this.add(RPT()),
    microservice: this.add(Microservice()),
    lpToken:      this.add(SNIP20('LPTOKEN')),
    rewards_v3:   this.add(Rewards('v3')),
    migrate:      this.add(h('x-button')),
    rewards_v4:   this.add(Rewards('v4'))
  }

  constructor () {
    super()

    for (const contract of [this.ui.sienna, this.ui.lpToken]) {
      contract.addAccount('Admin')
      contract.addAccount('MGMT')
      contract.addAccount('RPT')
      contract.addAccount('Rewards V3')
      contract.addAccount('Rewards V4')
    }

    //for (let i = 0; i < 10; i++) {
      //const id = `User${i}`
      //this.sienna.add(id)
      //this.lpToken.add(id, Math.floor(Math.random()*100000))
      //this.rewards_v3.add(id)
      //this.rewards_v4.add(id)
    //}
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
  }
}

type Contracts = Record<string, any> 

customElements.define('x-dashboard', Dashboard)
export default function dashboard (contracts: Record<string, any>) {
  return h('x-dashboard', { contracts, className: 'Outside Dashboard' })
}

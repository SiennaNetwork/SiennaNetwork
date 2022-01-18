import type { IAgent } from '@fadroma/scrt'
import { workspace } from '@sienna/settings'
import { SNIP20Contract_1_0, SNIP20Executor } from '@fadroma/snip20'
import { randomHex } from '@hackbg/tools'
import { InitMsg } from './schema/init_msg.d'

export class SiennaSNIP20Contract extends SNIP20Contract_1_0 {

  crate = 'snip20-sienna'

  name  = 'SiennaSNIP20'

  initMsg: InitMsg = {
    name:      "Sienna",
    symbol:    "SIENNA",
    decimals:  18,
    config:    { public_total_supply: true },
    prng_seed: randomHex(36)
  }

  //tx (agent: IAgent = this.instantiator) {
    //return new SiennaSNIP20Executor(this, agent)
  //}

}

//export class SiennaSNIP20Executor extends SNIP20Executor {

  //constructor (
    //readonly contract: SiennaSNIP20Contract,
    //readonly agent:    IAgent
  //) {}

  //mint ({ amount, recipient, padding }: {
    //amount:    BigInt|String,
    //recipient: string,
    //padding:   string|null
  //}) {
    //const msg = { mint: { amount, recipient, padding } }
    //return this.agent.execute(this.contract, msg)
  //}

//}

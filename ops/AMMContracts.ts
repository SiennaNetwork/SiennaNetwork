import Ensemble from '@fadroma/scrt-ops/ensemble.js'
import { abs, args, combine } from './lib/index.js'

export default class AMMContracts extends Ensemble {

  workspace = abs()

  prefix = `${new Date().toISOString()} `

  contracts = {
    FACTORY: {
      crate: 'factory'
    },
    SNIP20: {
      crate: 'amm-snip20'
    },
    EXCHANGE: {
      crate: 'exchange'
    },
    LP_TOKEN: {
      crate: 'lp-token'
    },
    IDO: {
      crate: 'ido'
    }
  }

  get localCommands () {
    return [
      ["build",  '👷 Compile contracts from working tree',
        (context, [sequential]) => this.build(sequential)],
    ]
  }

  get remoteCommands () {
    return [
      ["deploy", '🚀 Build, init, and deploy the rewards component',
        (context, [x]) => this.deploy(x).then(console.info)]
    ]
  }

}

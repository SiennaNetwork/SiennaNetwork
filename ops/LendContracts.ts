import Ensemble from '@fadroma/scrt-ops/ensemble.js'
import { abs, args, combine } from './lib/index.js'

export default class AMMContracts extends Ensemble {

  workspace = abs()

  prefix = `${new Date().toISOString()} `

  contracts = {
    SNIP20: {
      crate: 'snip20-lend'
    },
    ATOKEN: {
      crate: 'atoken'
    },
    CONFIG: {
      crate: 'configuration'
    },
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

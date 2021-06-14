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

}

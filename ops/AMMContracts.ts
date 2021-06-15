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

}

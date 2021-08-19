import { Ensemble } from '@fadroma/ensemble'
import { abs } from './lib/index.js'

export default class SiennaLend extends Ensemble {

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

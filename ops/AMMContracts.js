import Ensemble from '@fadroma/scrt-ops/ensemble.js'
import { abs } from './root.js'
import { combine, args } from './args.js'

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

    commands (yargs) {
        return yargs
          .command('build-amm',
            '👷 Compile contracts from working tree',
            args.Sequential, () => this.build())
          .command('deploy-amm [network]',
            '🚀 Build, init, and deploy the rewards component',
            combine(args.Network),
            x => console.log('Use scripts in ops/swap/integration-tests/ instead.'))
      }
}

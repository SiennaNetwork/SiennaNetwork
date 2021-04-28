import { taskmaster, SecretNetwork } from '@hackbg/fadroma'
import { pull } from '@hackbg/fadroma/js/net.js'
import { fileURLToPath, resolve, basename, extname, dirname
       , readFile, writeFile } from '@hackbg/fadroma/js/sys.js'

export const __dirname = dirname(fileURLToPath(import.meta.url))
export const abs = (...args) => resolve(__dirname, ...args)
export const stateBase = abs('artifacts')

const prefix = new Date().toISOString().replace(/[-:\.]/g, '-').replace(/[TZ]/g, '_')
const prng_seed = 'insecure'

export const CONTRACTS =
  { CASHBACK:
    { crate: 'cashback' 
    , label: `${prefix}SIENNA.Rewards.Cashback` }
  , GOVERNANCE_TOKEN:
    { crate: 'gov-token'
    , label: `${prefix}SIENNA.Rewards.GovernanceToken` }
  , LP_STAKING:
    { crate: 'lp-staking'
    , label: `${prefix}SIENNA.Rewards.LPStaking` }
  , WEIGHT_MASTER:
    { crate: 'weight-master'
    , label: `${prefix}SIENNA.Rewards.WeightMaster` } }

export async function build (options = {}) {
  console.log('hi')
  const { task      = taskmaster()
        , builder   = new SecretNetwork.Builder()
        , workspace = __dirname
        , outputDir = resolve(workspace, 'artifacts') } = options

  // pull build container
  await pull('enigmampc/secret-contract-optimizer:latest')

  // build all contracts
  const binaries = {}
  await task.parallel('build project',
    ...Object.entries(CONTRACTS).map(([name, {crate}])=>
      task(`build ${name}`, async report => {
        binaries[name] = await builder.build({outputDir, workspace, crate})
      })))

  return binaries
}

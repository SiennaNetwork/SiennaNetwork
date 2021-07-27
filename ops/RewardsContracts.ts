import { execFileSync } from 'child_process'
import { Console, readFileSync, randomBytes, taskmaster } from '@fadroma/utilities'
import { ScrtEnsemble } from '@fadroma/scrt-ops'
import { SNIP20Contract, RewardsContract } from '@sienna/api'
import { abs, combine, args } from './lib/index.js'

const console = Console(import.meta.url)

export default class SiennaRewards extends ScrtEnsemble {

  workspace = abs()

  prefix = `${new Date().toISOString()}`

  // TODO: make tokens "pluggable"
  // i.e. allow attaching the rewards to an existing deployment
  contracts =
  { LP: { crate:   'amm-snip20'
        , label:   `${this.prefix}/lp_token`
        , initMsg: { prng_seed: randomBytes(36).toString('hex')
                   , name:     "LPToken"
                   , symbol:   "LP"
                   , decimals: 18
                   , config: { public_total_supply: true
                             , enable_deposit:      true
                             , enable_redeem:       true
                             , enable_mint:         true
                             , enable_burn:         true } } }
  , REWARD: { crate:   'amm-snip20'
            , label:   `${this.prefix}/reward_token`
            , initMsg: { prng_seed: randomBytes(36).toString('hex')
                       , name:     "RewardToken"
                       , symbol:   "REWARD"
                       , decimals: 18
                       , config: { public_total_supply: true
                                 , enable_deposit:      true
                                 , enable_redeem:       true
                                 , enable_mint:         true
                                 , enable_burn:         true } } }

  , POOL: { crate:   'sienna-rewards'
          , label:   `${this.prefix}/reward_pool`
          , initMsg: {}}

  , FACTORY: { crate:   'factory'
             , label:   `${this.prefix}/amm_factory`
             , initMsg: {}}

  , EXCHANGE: { crate:   'exchange'
              , label:   `${this.prefix}/amm_exchange`
              , initMsg: {}}
  }

  get localCommands () {
    return [ ...super.localCommands
           , ["test",      '🥒 Run unit tests',    this.test.bind(this)      ]
           , ["benchmark", '⛽ Measure gas costs', this.benchmark.bind(this) ] ]
  }

  test (context: object, ...args:any) {
    execFileSync('cargo', [
      'test', '-p', 'sienna-rewards', ...args
    ], {
      stdio: 'inherit'
    })
  }

  benchmark () {
    /* stupid esmodule import issue when running mocha programmatically
     * their CLI works fine though... нтар 🤦‍♂️
    const mocha = new Mocha()
    mocha.addFile(abs('api/RewardsBenchmark.spec.js'))
    mocha.run(fail => process.exit(fail ? 1 : 0))*/
    execFileSync(abs('node_modules/.bin/mocha'), [
      '-p', 'false', // what was that
      'api/RewardsBenchmark.spec.js'
    ], {
      stdio: 'inherit'
    })
  }

  async initialize (/*{ network, receipts, agent = network.agent }*/) {
    throw new Error('todo!')
    //const instances = {}
    //const task = taskmaster()

    //await task('initialize token', async report => {
      //const {codeId} = receipts.TOKEN
      //const {label, initMsg} = this.contracts.TOKEN
      //Object.assign(initMsg, {
        //admin: agent.address
      //})
      //instances.TOKEN = await agent.instantiate(new SNIP20Contract({codeId, label, initMsg}))
      //report(instances.TOKEN.transactionHash)
    //})

    //await task('initialize rewards', async report => {
      //const {codeId} = receipts.REWARDS
      //const {label, initMsg} = this.contracts.REWARDS
      //console.log(agent.address)
      //Object.assign(initMsg, {
        //admin:     agent.address,
        //entropy:   randomBytes(36).toString('base64'),
        //prng_seed: randomBytes(36).toString('base64'),
        //reward_token: instances.TOKEN.reference,
      //})
      //instances.REWARDS = await agent.instantiate(new RewardsContract({codeId, label, initMsg}))
      //report(instances.REWARDS.transactionHash)
    //})

    //await task('mint reward token', async report => {
      //const result = await instances.TOKEN.mint('540000000000000000000000', agent, instances.REWARDS.address)
      //report(result)
    //})

    //console.log(instances)

    //console.table([
      //['Contract\nDescription',      'Address\nCode hash'],
      //['TOKEN\nSienna SNIP20 token', `${instances.TOKEN.address}\n${instances.TOKEN.codeHash}`],
      //['Rewards\n',                  `${instances.REWARDS.address}\n${instances.REWARDS.codeHash}`],
    //])

    //return instances
  }

  /** Attach reward pool to existing deployment. */
  async augment () {
    throw new Error('todo!')
  }

}

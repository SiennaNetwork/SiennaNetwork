import { Console, readFileSync, randomBytes } from '@fadroma/utilities'
import Ensemble from '@fadroma/scrt-ops/ensemble.js'
import { SNIP20Contract, RewardsContract } from '@sienna/api'
import { abs } from './root.js'
import { combine, args } from './args.js'

const console = Console(import.meta.url)

export default class RewardsContracts extends Ensemble {

  workspace = abs()

  prefix = `${new Date().toISOString()} `

  contracts = {

    TOKEN: {
      crate: 'snip20-sienna',
      label: `${this.prefix}SIENNA_SNIP20`,
      initMsg: {
        prng_seed: randomBytes(36).toString('hex'),
        name:      "Sienna",
        symbol:    "SIENNA",
        decimals:  18,
        config:    { public_total_supply: true }
      }
    },

    REWARDS: {
      crate: 'sienna-rewards',
      label: `${this.prefix}SIENNA_REWARDS`,
      initMsg: JSON.parse(
        readFileSync(abs('settings/rewards.json'), 'utf8')
      )
    }

  }

  async initialize ({ receipts, agent }) {
    const instances = {}
    const task = taskmaster()

    await task('initialize token', async report => {
      const {codeId} = receipts.TOKEN
      const {label, initMsg} = this.contracts.TOKEN
      Object.assign(initMsg, {
        admin: agent.address
      })
      instances.TOKEN = await agent.instantiate(new SNIP20Contract({codeId, label, initMsg}))
      report(instances.TOKEN.transactionHash)
    })

    await task('initialize rewards', async report => {
      const {codeId} = receipts.REWARDS
      const {label, initMsg} = this.contracts.REWARDS
      console.log(agent.address)
      Object.assign(initMsg, {
        admin:     agent.address,
        entropy:   randomBytes(36).toString('base64'),
        prng_seed: randomBytes(36).toString('base64'),
        reward_token: {
          address:   instances.TOKEN.contractAddress,
          code_hash: instances.TOKEN.codeHash
        },
      })
      instances.REWARDS = await agent.instantiate(new RewardsContract({codeId, label, initMsg}))
      report(instances.REWARDS.transactionHash)
    })

    await task('mint reward token', async report => {
      const result = await instances.TOKEN.mint(agent, '390000000000000000000000', instances.REWARDS.contractAddress)
      report(result)
    })

    console.log(instances)

    console.table([
      ['Contract\nDescription',      'Address\nCode hash'],
      ['TOKEN\nSienna SNIP20 token', `${instances.TOKEN.address}\n${instances.TOKEN.codeHash}`],
      ['Rewards\n',                  `${instances.REWARDS.address}\n${instances.REWARDS.codeHash}`],
    ])

    return instances
  }

  commands (yargs) {
    return yargs
      .command('build-rewards',
        '👷 Compile contracts from working tree',
        args.Sequential, () => this.build())
      .command('deploy-rewards [network]',
        '🚀 Build, init, and deploy the rewards component',
        combine(args.Network),
        x => this.deploy(x).then(console.info))
  }

}

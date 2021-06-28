import { Console, readFileSync, randomBytes, taskmaster } from '@fadroma/utilities'
import Ensemble from '@fadroma/scrt-ops/ensemble.js'
import { SNIP20Contract, RewardsContract } from '@sienna/api'
import { abs, combine, args } from './lib/index.js'

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
    },

    REWARDS_FACTORY: {
      crate: 'sienna-rewards-factory',
      label: `${this.prefix}SIENNA_REWARDS_FACTORY`
    }
  }

  async initialize ({ network, receipts, agent = network.agent }) {
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
        reward_token: instances.TOKEN.reference,
      })
      instances.REWARDS = await agent.instantiate(new RewardsContract({codeId, label, initMsg}))
      report(instances.REWARDS.transactionHash)
    })

    await task('mint reward token', async report => {
      const result = await instances.TOKEN.mint('540000000000000000000000', agent, instances.REWARDS.address)
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

}

import { abs } from './index.js'
import { bold, render } from '@fadroma/utilities'
import { SecretNetwork } from '@fadroma/scrt-agent'
import * as repl from 'repl'
import * as vm from 'vm'

import {
  AMMContract,
  FactoryContract,
  IDOContract,
  MGMTContract,
  RPTContract,
  RewardsContract,
  SNIP20Contract,
} from '@sienna/api'

import TGE from '../TGEContracts.js'
import Rewards from '../RewardsContracts.ts'
import Swap from '../AMMContracts.ts'
import Lend from '../LendContracts.ts'

const Contracts = {
  AMM:     AMMContract,
  Factory: FactoryContract,
  IDO:     IDOContract,
  MGMT:    MGMTContract,
  RPT:     RPTContract,
  Rewards: RewardsContract,
  SNIP20:  SNIP20Contract
}
const Ensembles = {
  TGE,
  Rewards,
  Swap,
  Lend
}

export default async function shell (context = {}) {
  const {network, agent, builder} = await SecretNetwork[context.network]().connect()
  console.info(`Launching shell...`)
  if (Object.keys(Contracts).length > 0) {
    console.info(bold(`Available contracts:`))
    console.info('  '+Object.keys(Contracts).map(x=>`Contracts.${x}`).join('\n  '))
  }
  if (Object.keys(Ensembles).length > 0) {
    console.info(bold(`Available ensembles:`))
    console.info('  '+Object.keys(Ensembles).map(x=>`Ensembles.${x}`).join('\n  '))
  }
  console.info(bold(`Other entities:`))
  console.info('  '+['network', 'agent', 'builder', 'workspace'].join('\n  '))
  const loop = repl.start({
    prompt: `${context.network}> `,
    //writer: x => render(x),
    async eval (cmd, context, filename, callback) {
      try {
        const result = await Promise.resolve(vm.runInContext(cmd, context))
        return callback(null, result)
      } catch (e) {
        console.error(e)
        return callback()
      }
    },
  })
  await new Promise((resolve, reject)=>
    loop.setupHistory('.fadroma_repl_history',
      (err, repl) => err ? reject(err) : resolve(repl)))
  Object.assign(loop.context, { Contracts, Ensembles, network, agent, builder, workspace: abs() })
}

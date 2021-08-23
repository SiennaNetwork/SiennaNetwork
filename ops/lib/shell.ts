import * as repl from 'repl'
import * as vm from 'vm'

import { Scrt, bold } from '@hackbg/fadroma'
import { abs } from './root'

import {
  AMMContract,
  FactoryContract,
  IDOContract,
  MGMTContract,
  RPTContract,
  RewardsContract,
  SNIP20Contract,
} from '@sienna/api'

import {
  SiennaTGE     as TGE,
  SiennaSwap    as Swap,
  SiennaRewards as Rewards,
  SiennaLend    as Lend } from '../ensembles'

const Contracts = {
  AMM:     AMMContract,
  Factory: FactoryContract,
  IDO:     IDOContract,
  MGMT:    MGMTContract,
  RPT:     RPTContract,
  Rewards: RewardsContract,
  SNIP20:  SNIP20Contract }

const Ensembles = {
  TGE,
  Rewards,
  Swap,
  Lend }

export async function shell (context: any) {
  const {chain, agent, builder} = await Scrt[context.chain]().connect()
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
  console.info('  '+['chain', 'agent', 'builder', 'workspace'].join('\n  '))
  const loop = repl.start({
    prompt: `${context.chain}> `,
    //writer: x => render(x),
    async eval (cmd, context, _, callback) {
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
  Object.assign(loop.context, { Contracts, Ensembles, chain, agent, builder, workspace: abs() }) }

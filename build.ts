/// # Sienna Development

import { resolve, dirname } from 'path'
import { fileURLToPath } from 'url'
import { execFileSync } from 'child_process'
import process from 'process'
import { bold } from '@fadroma/tools'
import { schemaToTypes } from '@fadroma/scrt'
import { cargo } from '@fadroma/tools'
import {
  SiennaSNIP20,
  MGMTContract,
  RPTContract,
  FactoryContract,
  AMMContract,
  AMMSNIP20,
  LPToken,
  RewardsContract,
  IDOContract,
  LaunchpadContract,
  SwapRouterContract,
} from '@sienna/api'

import rewardsBenchmark from './benchmarks/rewards'

const
  projectRoot = resolve(dirname(fileURLToPath(import.meta.url))),
  abs         = (...args: Array<string>) => resolve(projectRoot, ...args)


/// ## Entry point


if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(()=>process.exit(0))
}

export default async function main (words: Array<string>) {


  const commands = {

    build: {

      all: () => Promise.all([
        new SiennaSNIP20().build(),
        new MGMTContract().build(),
        new RPTContract().build(),
        new AMMContract().build(),
        new AMMSNIP20().build(),
        new LPToken().build(),
        new FactoryContract().build(),
        new RewardsContract().build(),
        new IDOContract().build(),
        new LaunchpadContract().build(),
        new SwapRouterContract().build()
      ]),

      tge: () => Promise.all([
        new SiennaSNIP20().build(),
        new MGMTContract().build(),
        new RPTContract().build()
      ]),

      swap: () => Promise.all([
        new AMMContract().build(),
        new AMMSNIP20().build(),
        new LPToken().build(),
        new SwapRouterContract().build(),
        new FactoryContract().build(),
      ]),

      rewards: () => Promise.all([
        new RewardsContract().build(),
      ]),

      ido: () => Promise.all([
        new IDOContract().build(),
        new LaunchpadContract().build()
      ])

    },

    test: {
      tge: testCommandsFor(
        'snip20-sienna', 'sienna-mgmt', 'sienna-rpt'
      ),
      swap: testCommandsFor(
        'factory', 'exchange', 'lp-token', 'amm-snip20', 'router'
      ),
      rewards: testCommandsFor(
        'sienna-rewards'
      ),
      ido: testCommandsFor(
        'launchpad', 'ido'
      )
    },

    async schema () {
      cargo('run', '--bin', 'schema')
      await schemaToTypes(...[
        'amm/handle_msg.json',
        'amm/init_msg.json',
        'amm/query_msg.json',
        'amm/query_msg_response.json',
        'amm/receiver_callback_msg.json',
        'factory/handle_msg.json',
        'factory/init_msg.json',
        'factory/query_msg.json',
        'factory/query_response.json',
        'ido/handle_msg.json',
        'ido/init_msg.json',
        'ido/query_msg.json',
        'ido/query_response.json',
        'ido/receiver_callback_msg.json',
        'launchpad/handle_msg.json',
        'launchpad/init_msg.json',
        'launchpad/query_msg.json',
        'launchpad/query_response.json',
        'launchpad/receiver_callback_msg.json',
        'mgmt/handle.json',
        'mgmt/init.json',
        'mgmt/query.json',
        'mgmt/response.json',
        'rewards/handle.json',
        'rewards/init.json',
        'rewards/query.json',
        'rewards/response.json',
        'router/handle_msg.json',
        'router/init_msg.json',
        'router/query_msg.json',
        'rpt/handle.json',
        'rpt/init.json',
        'rpt/query.json',
        'rpt/response.json',
        'snip20/handle_answer.json',
        'snip20/handle_msg.json',
        'snip20/init_msg.json',
        'snip20/query_answer.json',
        'snip20/query_msg.json'
      ].map(x=>abs('api', x)))
    },

    bench: {
      rewards: rewardsBenchmark,
      ido:     notImplemented
    },

    demo: {
      tge:     notImplemented,
      rewards: notImplemented
    }

  }

  return await runCommands(words, commands)


  /// Command parser


  async function runCommands (
    words: Array<string>,
    commands: Record<string, any>
  ) {
    let command = commands
    let i: number
    for (i = 0; i < words.length; i++) {
      const word = words[i]
      if (typeof command === 'object' && command[word]) command = command[word]
      if (command instanceof Function) break
    }
    if (command instanceof Function) {
      return await Promise.resolve(command(...words.slice(i + 1)))
    } else {
      console.log(`\nAvailable commands:`)
      for (const key of Object.keys(command)) {
        console.log(`  ${bold(key)}`)
      }
    }
  }


  /// For missing commands


  function notImplemented () {
    console.log('This command is not implemented yet.')
    process.exit(1)
  }


  function testCommandsFor (...crates: Array<string>) {

    const commands: Record<string, any> = {}

    if (crates.length > 1) {
      commands['all'] = (...argv: Array<string>) => {
        for (const crate of crates) {
          testCrate(crate, argv)
        }
      }
    }

    for (const crate of crates) {
      commands[crate] = function (...argv: Array<string>) {
        testCrate(crate, argv)
      }
    }

    function testCrate (crate: string, argv: Array<string>) {
      const args = ['test', '-p', crate, ...argv]
      const {status} = execFileSync('cargo', args, {
        stdio: 'inherit',
        env: { ...process.env, RUST_BACKTRACE: 'full' }
      })
      if (status !== 0) process.exit(status)
    }

  }


}

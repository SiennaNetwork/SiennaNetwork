/// # Sienna Development

import { resolve } from 'path'
import { fileURLToPath } from 'url'
import { execFileSync } from 'child_process'
import {
  readdirSync,
  readFileSync
} from 'fs'
import process from 'process'

import TOML from 'toml'

import { bold }          from '@fadroma/tools'
import { schemaToTypes } from '@fadroma/scrt'
import { cargo }         from '@fadroma/tools'

import { abs } from '@sienna/settings'

import {
  SiennaSNIP20Contract,
  MGMTContract,
  RPTContract,
  FactoryContract,
  AMMContract,
  AMMSNIP20Contract,
  LPTokenContract,
  RewardsContract,
  IDOContract,
  LaunchpadContract,
  SwapRouterContract,
} from '@sienna/api'

import { rewardsBenchmark } from '@sienna/benchmarks'


/// ## Entry point


if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(()=>process.exit(0))
}

export default async function main (words: Array<string>) {

  const commands = {

    build: {

      all: () => Promise.all([
        new SiennaSNIP20Contract().build(),
        new MGMTContract().build(),
        new RPTContract().build(),
        new AMMContract().build(),
        new AMMSNIP20Contract().build(),
        new LPTokenContract().build(),
        new FactoryContract().build(),
        new RewardsContract().build(),
        new IDOContract().build(),
        new LaunchpadContract().build(),
        new SwapRouterContract().build()
      ]),

      tge: () => Promise.all([
        new SiennaSNIP20Contract().build(),
        new MGMTContract().build(),
        new RPTContract().build()
      ]),

      swap: () => Promise.all([
        new AMMContract().build(),
        new AMMSNIP20Contract().build(),
        new LPTokenContract().build(),
        new SwapRouterContract().build(),
        new FactoryContract().build()
      ]),

      rewards: () => Promise.all([
        new RewardsContract().build(),
      ]),

      ido: () => Promise.all([
        new IDOContract().build(),
        new LaunchpadContract().build()
      ])

    },

    test: () => {
      //tge: testCommandsFor(
        //'snip20-sienna', 'sienna-mgmt', 'sienna-rpt'
      //),
      //swap: testCommandsFor(
        //'factory', 'exchange', 'lp-token', 'amm-snip20'
      //),
      //rewards: testCommandsFor(
        //'sienna-rewards'
      //),
      //ido: testCommandsFor(
        //'launchpad', 'ido'
      //)
      console.log(`\nThis command is on vacation. Please use "cargo test -p $CRATE" till it's back\n🌴 ⛱️  🐬\n`)
      process.exit(42)
    },

    async schema () {
      for (const dir of [
        "amm-snip20",
        "exchange",
        "factory",
        "ido",
        "launchpad",
        "lp-token",
        "mgmt",
        "rewards",
        "router",
        "rpt",
        "snip20-sienna",
      ]) {

        // Generate JSON schema
        const cargoToml = abs('contracts', dir, 'Cargo.toml')
        const {package:{name}} = TOML.parse(readFileSync(cargoToml, 'utf8'))
        cargo('run', '-p', name, '--example', 'schema')

        // Generate type definitions from JSON schema
        const schemaDir = abs('contracts', dir, 'schema')
        const schemas   = readdirSync(schemaDir).filter(x=>x.endsWith('.json'))
        await schemaToTypes(...schemas.map(x=>resolve(schemaDir, x)))
      }
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

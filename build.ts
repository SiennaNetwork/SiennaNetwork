/// # Sienna Development


import { fileURLToPath } from 'url'
import { execFileSync } from 'child_process'
import { bold } from '@fadroma/tools'
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
  LaunchpadContract
} from '@sienna/api'


/// ## Entry point


if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(()=>process.exit(0))
}

export default async function main (words: Array<string>) {


  const commands = {

    build: {

      tge: () => Promise.all([
        new SiennaSNIP20().build(),
        new MGMTContract().build(),
        new RPTContract().build()
      ]),

      swap: () => Promise.all([
        new AMMContract().build(),
        new AMMSNIP20().build(),
        new LPToken().build(),
        new FactoryContract().build()
      ]),

      rewards: () => Promise.all([
        new RewardsContract().build()
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
        'factory', 'exchange', 'lp-token', 'amm-snip20'
      ),
      rewards: testCommandsFor(
        'sienna-rewards'
      ),
      ido: testCommandsFor(
        'launchpad', 'ido'
      )
    },

    bench: {
      rewards: notImplemented,
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

#!/usr/bin/env node
// core
import { readFileSync, writeFileSync, existsSync, readdirSync, statSync } from 'fs'
import { resolve, basename, extname, dirname } from 'path'
import { env, argv, stdout, stderr, exit } from 'process'
import { execFileSync } from 'child_process'
import { fileURLToPath } from 'url'

// 3rd party
import { render } from 'prettyjson'
import open from 'open'
import yargs from 'yargs'

// custom
import { SecretNetwork } from '@hackbg/fadroma'
import { scheduleFromSpreadsheet } from '@hackbg/schedule'
import { build, upload, initialize } from './ops.js'
import demo from './demo.js'

// resolve path relative to this file's parent directory
const abs = (...args) =>
  resolve(dirname(fileURLToPath(import.meta.url)), ...args)

// [contracts that can be built] -> [`cargo run --example` target to generate JSON schema]
const CONTRACTS = {
  'token': {
    packageName:     'snip20-reference-impl',
    schemaGenerator: 'schema'
  },
  'mgmt': {
    packageName:     'sienna-mgmt',
    schemaGenerator: 'mgmt_schema'
  },
  'rpt': {
    packageName:     'sienna-rpt',
    schemaGenerator: 'rpt_schema'
  }
}

yargs(process.argv.slice(2))
  .wrap(yargs().terminalWidth())
  .demandCommand(1, '') // print usage by default

  .command('docs [crate]',
    '📖 Build the documentation and open it in a browser.',
    yargs => yargs.positional('crate', {
      describe: 'path to input file',
      default: 'sienna_schedule'
    }),
    function docs ({crate}) {
      const target = abs('target', 'doc', crate, 'index.html')
      try {
        stderr.write(`⏳ Building documentation...\n\n`)
        cargo('doc')
      } catch (e) {
        stderr.write('\n🤔 Building documentation failed.')
        if (existsSync(target)) {
          stderr.write(`\n⏳ Opening what exists at ${target}...`)
        } else {
          return
        }
      }
      open(`file:///${target}`)
    })

  .command('test',
    '⚗️  Run test suites for all the individual components.',
    function runTests () {
      clear()
      stderr.write(`⏳ Running tests...\n\n`)
      try {
        run('sh', '-c',
          'cargo test --color=always --no-fail-fast -- --nocapture --test-threads=1 2>&1'+
          ' | less -R')
        stderr.write('\n🟢 Tests ran successfully.\n')
      } catch (e) {
        stderr.write('\n👹 Tests failed.\n')
      }
    })

  .command('coverage',
    '🗺️  Generate test coverage and open it in a browser.',
    function generateCoverage () {
      // fixed by https://github.com/rust-lang/cargo/issues/9220
      let output = abs('docs', 'coverage')
      cargo('tarpaulin', '--out=Html', `--output-dir=${output}`)
        //'tarpaulin', 
        //'--avoid-cfg-tarpaulin', // ???
        //'--workspace', // obviously
        //'--no-fail-fast', // try to continue on test failure
        //'--verbose', // why not
        //'-o', 'Html', // output as html
        //`--exclude-files=${resolve(__dirname, 'libraries', 'platform')}`, // ignore vendor libs
        //`--output-dir=${output}`
      //)
    })

  .command('demo [script] [--testnet]',
    '📜 Run integration tests/demos/executable reports.',
    yargs => yargs.positional('script', {
      describe: 'path to demo script',
      default: 'demo.mjs'
    }).option('testnet', {
      describe: 'run on holodeck-2 instead of locally'
    }),
    async function runDemo ({script, testnet}) {
      clear()
      //script = abs('integration', script)
      stderr.write(`⏳ Running demo ${script}...\n\n`)
      try {
        if (testnet) {
          throw new Error('not implemented')
        } else {
          const stateBase = resolve(dirname(fileURLToPath(import.meta.url)), '.fadroma')
          const environment = await SecretNetwork.localnet({stateBase})
          await demo(environment)
          stderr.write('\n🟢 Demo executed successfully.\n')
        }
      } catch (e) {
        console.error(e)
        stderr.write('\n👹 Demo failed.\n')
      }
    })

  .command('schema',
    `🤙 Regenerate JSON schema for each contract's API.`,
    function schema () {
      const cwd = process.cwd()
      try {
        for (const [name, {schemaGenerator}] of Object.entries(CONTRACTS)) {
          const contractDir = abs('contracts', name)
          stderr.write(`Generating schema in ${contractDir}...`)
          process.chdir(contractDir)
          cargo('run', '--example', schemaGenerator)
        }
      } finally {
        process.chdir(cwd)
      }
    })

  .command('schedule [file]',
    '📅 Convert a spreadsheet into a JSON schedule for the contract.',
    yargs => yargs.positional('spreadsheet', {
      describe: 'path to input spreadsheet',
      default: abs('settings', 'schedule.ods')
    }),
    function configure ({ file }) {
      file = resolve(file)
      stderr.write(`⏳ Importing configuration from ${file}...\n\n`)
      const name = basename(file, extname(file)) // path without extension
      const schedule = scheduleFromSpreadsheet({ file })
      const serialized = stringify(schedule)
      stderr.write(render(JSON.parse(serialized))) // or `BigInt`s don't show
      const output = resolve(dirname(file), `${name}.json`)
      stderr.write(`\n\n⏳ Saving configuration to ${output}...\n\n`)
      writeFileSync(output, stringify(schedule), 'utf8')
      stderr.write(`🟢 Configuration saved to ${output}`)
    })

  .command('build [ref]',
    '👷 Compile contracts from working tree',
    () => build({ builder: new SecretNetwork.Builder() }))

  .command('deploy',
    '🚀 Upload, instantiate, and configure all contracts.',
    function deploy () {
      stderr.write('\nNot implemented.')
      exit(0)
    })

  .command('launch',
    '💸 Launch the vesting contract.',
    function deploy () {
      stderr.write('\nNot implemented.')
      exit(0)
    })

  .argv

function stringify (data) {
  const indent = 2
  const withBigInts = (k, v) => typeof v === 'bigint' ? v.toString() : v
  return JSON.stringify(data, withBigInts, indent)
}

function cargo (...args) {
  run('cargo', '--color=always', ...args)
}

function clear () {
  if (env.TMUX) {
    run('sh', '-c', 'clear && tmux clear-history')
  }
}

function run (cmd, ...args) {
  stderr.write(`\n🏃 running:\n${cmd} ${args.join(' ')}\n\n`)
  execFileSync(cmd, [...args], {stdio:'inherit'})
}

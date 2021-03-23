#!/usr/bin/env node

import { readFileSync, writeFileSync, existsSync, readdirSync, statSync } from 'fs'
import { resolve, basename, extname, dirname } from 'path'
import { env, argv, stdout, stderr, exit } from 'process'
import { execFileSync } from 'child_process'
import { fileURLToPath } from 'url'

import { render } from 'prettyjson'
import open from 'open'
import yargs from 'yargs'

import { buildCommit, buildWorkingTree } from '@hackbg/fadroma/js/builder.js'
import { scheduleFromSpreadsheet } from '@hackbg/schedule'

const abs = (...args) => resolve(dirname(fileURLToPath(import.meta.url)), ...args)

yargs(process.argv.slice(2))
  .demandCommand(1, '') // print usage by default

  .command('docs [crate]',
    'Build Rust documentation and open it in a browser.',
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
    'Run test suites for entire Cargo workspace.',
    function runTests () {
      clear()
      stderr.write(`⏳ Running tests...\n\n`)
      try {
        cargo('test')
        stderr.write('\n🟢 Tests ran successfully.\n')
      } catch (e) {
        stderr.write('\n👹 Tests failed.\n')
      }
    })

  .command('demo',
    'Run one of the integration tests / executable reports.',
    function runDemo ({demo = '002_rpt.mjs'}) {
      clear()
      demo = abs('integration', demo)
      stderr.write(`⏳ Running demo ${demo}...\n\n`)
      try {
        run('docker-compose', 'up', '-d', 'localnet')
        run('node', '--trace-warnings', '--unhandled-rejections=strict', demo)
        stderr.write('\n🟢 Demo executed successfully.\n')
      } catch (e) {
        stderr.write('\n👹 Demo failed.\n')
      }
    })

  .command('coverage',
    'Generate test coverage for the entire Cargo workspace and open it in a browser.',
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

  .command('configure [spreadsheet]',
    'Convert a spreadsheet into a JSON schedule for the contract.',
    yargs => yargs.positional('spreadsheet', {
      describe: 'path to input spreadsheet',
      default: abs('settings', 'schedule.ods')
    }),
    function configure ({ spreadsheet }) {
      spreadsheet = resolve(spreadsheet)
      stderr.write(`⏳ Importing configuration from ${spreadsheet}...\n\n`)
      const name = basename(spreadsheet, extname(spreadsheet)) // path without extension
      const schedule = scheduleFromSpreadsheet({ file })
      const serialized = stringify(schedule)
      stderr.write(render(JSON.parse(serialized))) // or `BigInt`s don't show
      const output = resolve(dirname(file), `${name}.json`)
      stderr.write(`\n\n⏳ Saving configuration to ${output}...\n\n`)
      writeFileSync(output, stringify(schedule), 'utf8')
      stderr.write(`🟢 Configuration saved to ${output}`)
    })

  .command('build [commit]',
    'Compiles production builds of all contracts '+
    'from either the working tree or a specific commit.',
    yargs => yargs.positional('commit', {
      describe: 'upstream commit to build'
    }),
    async function build ({ commit }) {
      const optimizer = abs('build', 'optimizer')
      run('docker', 'build',
        '--file=' + resolve(optimizer, 'Dockerfile'),
        '--tag=hackbg/secret-contract-optimizer:latest',
        optimizer)
      const buildOutputs = abs('build', 'outputs')
      Promise.all(readdirSync(abs('contracts'))
        .filter(x=>statSync(abs('contracts', x)).isDirectory())
        .map(name=>commit
            ? buildCommit({ commit, name, buildOutputs })
            : buildWorkingTree({ projectRoot: abs(), name, buildOutputs }))) })

  .command('deploy',
    'Deploys and configures all contracts.',
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

#!/usr/bin/env node
import { env, argv, stdout, stderr, exit } from 'process'
import { execFileSync } from 'child_process'
import yargs from 'yargs'
import { build } from './ops.js'

export default function main () {
  return yargs(process.argv.slice(2))
    .wrap(yargs().terminalWidth())
    .demandCommand(1, '')

    // validation:
    .command('test',
      '⚗️  Run test suites for all the individual components.',
      runTests)

    // artifacts:
    .command('build',
      '👷 Compile contracts from working tree',
      build)

    .argv
}

const runTests = () => {
  clear()
  stderr.write(`⏳ Running tests...\n\n`)
  try {
    run('sh', '-c',
      'cargo test --color=always --no-fail-fast -- --nocapture --test-threads=1 2>&1'+
      ' | less -R -F -m -Otest.log -s -w')
    stderr.write('\n🟢 Tests ran successfully.\n')
  } catch (e) {
    stderr.write('\n🔴 Tests failed.\n')
  }
}

export const run = (cmd, ...args) => {
  stderr.write(`\n🏃 running:\n${cmd} ${args.join(' ')}\n\n`)
  execFileSync(cmd, [...args], {stdio:'inherit'})
}

export const clear = () =>
  env.TMUX && run('sh', '-c', 'clear && tmux clear-history')

try {
  main()
} catch (e) {
  console.error(e)
  const ISSUES = `https://github.com/hackbg/sienna-secret-token/issues`
  console.info(`👹 That was a bug. Report it at ${ISSUES}`)
}

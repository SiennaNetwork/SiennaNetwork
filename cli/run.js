import { execFileSync } from 'child_process'
import { env, stderr } from 'process'

import {SecretNetwork} from '@fadroma/scrt-agent'

import { stateBase } from './ops.js'
import demo from './demo.js'

export const clear = () =>
  env.TMUX && run('sh', '-c', 'clear && tmux clear-history')

export const cargo = (...args) =>
  run('cargo', '--color=always', ...args)

export const run = (cmd, ...args) => {
  stderr.write(`\n🏃 running:\n${cmd} ${args.join(' ')}\n\n`)
  return execFileSync(cmd, [...args], {stdio:'inherit'})
}

export const outputOf = (cmd, ...args) => {
  stderr.write(`\n🏃 running:\n${cmd} ${args.join(' ')}\n\n`)
  return String(execFileSync(cmd, [...args]))
}

export const runTests = () => {
  clear()
  stderr.write(`⏳ Running tests...\n\n`)
  try {
    run('sh', '-c',
      'cargo test --color=always --no-fail-fast -- --nocapture --test-threads=1 2>&1'+
      ' | less -R')
    stderr.write('\n🟢 Tests ran successfully.\n')
  } catch (e) {
    stderr.write('\n🔴 Tests failed.\n')
  }
}

export const runDemo = async ({testnet}) => {
  clear()
  //script = abs('integration', script)
  try {
    let environment
    if (testnet) {
      console.info(`⏳ running demo on testnet...`)
      environment = await SecretNetwork.testnet({stateBase})
    } else {
      console.info(`⏳ running demo on localnet...`)
      environment = await SecretNetwork.localnet({stateBase})
    }
    await demo(environment)
    console.info('\n🟢 Demo executed successfully.\n')
  } catch (e) {
    console.error(e)
    console.info('\n🔴 Demo failed.\n')
  }
}

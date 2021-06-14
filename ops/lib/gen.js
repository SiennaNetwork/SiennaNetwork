import open from 'open'
import { existsSync, stderr, writeFileSync } from '@fadroma/utilities'
import { abs } from './root.js'
import { cargo } from './run.js'

export function genCoverage () {
  // fixed by https://github.com/rust-lang/cargo/issues/9220
  cargo('tarpaulin', '--out=Html', `--output-dir=${abs('docs', 'coverage')}`)
}

export function genSchema () {
  throw new Error('not implemented')
  //const cwd = process.cwd()
  //try {
    //for (const [name, {schema}] of Object.entries(TGEContracts.contracts)) {
      //const contractDir = abs('contracts', name.toLowerCase() [>!!!<] )
      //stderr.write(`Generating schema in ${contractDir}...`)
      //process.chdir(contractDir)
      //cargo('run', '--example', schema)
    //}
  //} finally {
    //process.chdir(cwd)
  //}
}

export function genDocs (context, crate = '', dontOpen = false) {
  const entryPoint = crate
    ? abs('target', 'doc', crate, 'index.html')
    : abs('target', 'doc')

  try {
    stderr.write(`⏳ Building documentation...\n\n`)
    cargo('doc')
  } catch (e) {
    stderr.write('\n🤔 Building documentation failed.')
    if (!dontOpen) {
      if (existsSync(entryPoint)) {
        stderr.write(`\n⏳ Opening what exists at ${entryPoint}...`)
      } else {
        stderr.write(`\n⏳ ${entryPoint} does not exist, opening nothing.`)
        return
      }
    }
  }

  if (!dontOpen) {
    open(`file://${entryPoint}`)
  }
}

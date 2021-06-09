import { existsSync, stderr, writeFileSync } from '@fadroma/utilities'
import TGEContracts from './TGEContracts.js'
import { abs } from './root.js'
import { cargo } from './run.js'

export function genCoverage () {
  // fixed by https://github.com/rust-lang/cargo/issues/9220
  cargo('tarpaulin', '--out=Html', `--output-dir=${abs('docs', 'coverage')}`)
}

export function genSchema () {
  const cwd = process.cwd()
  try {
    for (const [name, {schema}] of Object.entries(TGEContracts.contracts)) {
      const contractDir = abs('contracts', name.toLowerCase() /*!!!*/ )
      stderr.write(`Generating schema in ${contractDir}...`)
      process.chdir(contractDir)
      cargo('run', '--example', schema)
    }
  } finally {
    process.chdir(cwd)
  }
}

export function genDocs ({ crate }) {
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
}

import { writeFileSync } from 'fs'
import { resolve, basename, extname, dirname, existsSync } from '@fadroma/utilities/sys.js'
import { scheduleFromSpreadsheet } from '@sienna/schedule'

import { abs } from './root.js'
import { cargo } from './run.js'
import TGEContracts from './TGEContracts.js'

const {stderr} = process

const stringify = data => {
  const indent = 2
  const withBigInts = (k, v) => typeof v === 'bigint' ? v.toString() : v
  return JSON.stringify(data, withBigInts, indent)
}

export function genConfig (options = {}) {
  const { file = abs('settings', 'schedule.ods')
        } = options

  stderr.write(`\n⏳ Importing configuration from ${file}...\n\n`)
  const name       = basename(file, extname(file)) // path without extension
  const schedule   = scheduleFromSpreadsheet({ file })
  const serialized = stringify(schedule)
  const output     = resolve(dirname(file), `${name}.json`)
  stderr.write(`⏳ Saving configuration to ${output}...\n\n`)

  writeFileSync(output, stringify(schedule), 'utf8')
  stderr.write(`🟢 Configuration saved to ${output}\n`)
}

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

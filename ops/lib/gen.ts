import open from 'open'

import { scheduleFromSpreadsheet } from '@sienna/schedule'

import { stderr, existsSync, readFileSync, writeFileSync,
         resolve, basename, extname, dirname, } from '@fadroma/util-sys'

import { abs, projectRoot } from './root'
import { cargo } from './run'

export function genCoverage () {
  // fixed by https://github.com/rust-lang/cargo/issues/9220
  cargo('tarpaulin', '--out=Html', `--output-dir=${abs()}`, '--locked', '--frozen') }

export function genSchema () {
  cargo('run', '--bin', 'schema') }

export function genDocs (context, crate = '', dontOpen = false) {
  const entryPoint = crate
    ? abs('target', 'doc', crate, 'index.html')
    : abs('target', 'doc')
  try {
    stderr.write(`⏳ Building documentation...\n\n`)
    cargo('doc') }
  catch (e) {
    stderr.write('\n🤔 Building documentation failed.')
    if (!dontOpen) {
      if (existsSync(entryPoint)) {
        stderr.write(`\n⏳ Opening what exists at ${entryPoint}...`) }
      else {
        stderr.write(`\n⏳ ${entryPoint} does not exist, opening nothing.`)
        return } } }
  if (!dontOpen) {
    open(`file://${entryPoint}`) } }

export function getDefaultSchedule () {
  const path = resolve(projectRoot, 'settings', 'schedule.json')
  try {
    return JSON.parse(readFileSync(path, 'utf8')) }
  catch (e) {
    console.warn(`${path} does not exist - "./sienna.js config" should create it`)
    return null } }

export function genConfig (
  { file = abs('settings', 'schedule.ods') } = {}
) {
  stderr.write(`\n⏳ Importing configuration from ${file}...\n\n`)
  const name     = basename(file, extname(file)) // path without extension
      , schedule = scheduleFromSpreadsheet({ file })
      , output   = resolve(dirname(file), `${name}.json`)
  stderr.write(`⏳ Saving configuration to ${output}...\n\n`)
  writeFileSync(output, stringify(schedule), 'utf8')
  stderr.write(`🟢 Configuration saved to ${output}\n`) }

function stringify (data) {
  const indent = 2
  const withBigInts = (k, v) => typeof v === 'bigint' ? v.toString() : v
  return JSON.stringify(data, withBigInts, indent) }

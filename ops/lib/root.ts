import { resolve, dirname, fileURLToPath } from '@hackbg/fadroma'

export const projectRoot = resolve(
  dirname(fileURLToPath(import.meta.url)), '../..')

export const abs = (...args) => resolve(
  projectRoot, ...args)

export const stateBase = abs('artifacts')

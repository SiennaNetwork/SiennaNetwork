import { resolve, dirname, fileURLToPath } from '@fadroma/utilities/sys.js'

export const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')

export const abs = (...args) => resolve(projectRoot, ...args)

export const stateBase = abs('artifacts')

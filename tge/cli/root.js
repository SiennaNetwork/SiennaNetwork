import { resolve, dirname, fileURLToPath } from '@hackbg/fadroma/js/sys.js'

export const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')

export const abs = (...args) => resolve(projectRoot, ...args)

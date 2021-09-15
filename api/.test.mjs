import Mocha from 'mocha'
import {resolve, dirname} from 'path'
import { fileURLToPath } from 'url'
import fs from 'fs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), './')
const mocha = new Mocha()

let args = process.argv.slice(2, process.argv.length)

if (!args.length) {
  args = fs.readdirSync(root).filter(f => f.endsWith('spec.js') || f.endsWith('spec.mjs'));
}

for (const a of args) {
  mocha.addFile(resolve(root, a))
}

mocha.loadFilesAsync()
  .then(() => {
    mocha.run(failures => process.exitCode = failures ? 1 : 0) })
  .catch(e => {
    console.error(e)
    process.exitCode = 1 })

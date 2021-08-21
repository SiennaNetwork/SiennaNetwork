import { Scrt } from '@fadroma/agent'
import { basename, resolve, readdirSync, readFile } from '@fadroma/sys'
import { bold, table, noBorders } from '@fadroma/cli'

export default async function printStatus ({network}) {
  const { receipts, instances } = Scrt.hydrate(network)

  const idToName = {}

  const uploadReceipts = [[
    bold('  code id'), bold('name\n'), bold('size'), bold('hash')
  ]].concat(await Promise.all(readdirSync(receipts).map(async x=>{
    x = resolve(receipts, x)
    const { codeId,
            originalSize, compressedSize,
            originalChecksum, compressedChecksum } = JSON.parse(await readFile(x))
    const name = idToName[codeId] = basename(x, '.upload.json')
    return [
      `  ${codeId}`,
      `${bold(name)}\ncompressed:\n`,
      `${originalSize}\n${String(compressedSize).padStart(String(originalSize).length)}`,
      `${originalChecksum}\n${compressedChecksum}`]})))

  if (uploadReceipts.length > 1) {
    console.log(`\nUploaded binaries on ${bold(network)}:`)
    console.log('\n'+table(uploadReceipts, noBorders)) }
  else {
    console.log(`\n  No known uploaded binaries on ${bold(network)}`) }

  const initReceipts = [[
    bold('  label')+'\n  address', '(code id) binary name\ncode hash\ninit tx\n'
  ]].concat(await Promise.all(readdirSync(instances).map(async x=>{
    x = resolve(instances, x)
    const name = basename(x, '.json')
    const {codeId, codeHash, initTx} = await JSON.parse(await readFile(x))
    const {contractAddress, transactionHash} = initTx
    return [
      `  ${bold(name)}\n  ${contractAddress}`,
      `(${codeId}) ${idToName[codeId]||''}\n${codeHash}\n${transactionHash}\n`,
      /*`${contractAddress}\n${transactionHash}`*/ ] })))

  if (initReceipts.length > 1) {
    console.log(`Instantiated contracts on ${bold(network)}:`)
    console.log('\n'+table(initReceipts, noBorders)) }
  else {
    console.log(`\n  No known contracts on ${bold(network)}`) } }

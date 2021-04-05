import SNIP20Contract from '@hackbg/snip20'
import MGMTContract from '@hackbg/mgmt'
import RPTContract from '@hackbg/rpt'

import { taskmaster } from '@hackbg/fadroma'
import { resolve, dirname, fileURLToPath } from '@hackbg/fadroma/js/sys.js'
import { pull } from '@hackbg/fadroma/js/net.js'

export default async function deploy ({
  task     = taskmaster(),
  builder  = new SecretNetwork.Builder(),
  initMsgs
}) {
  return await task('build, upload, and initialize contracts', async () => {
    const binaries = await build({ task, builder })
    const receipts = await upload({ task, builder, binaries })
    const contracts = await initialize({ task, builder, initMsgs })
  })
}

export async function build ({
  task      = taskmaster(),
  workspace = resolve(dirname(fileURLToPath(import.meta.url))),
  outputDir = resolve(workspace, 'artifacts'),
  builder   = new SecretNetwork.Builder(),
} = {}) {
  await pull('enigmampc/secret-contract-optimizer:latest')
  const binaries = {}
  await task.parallel('build project',
    task('build token', async () => {
      binaries.TOKEN = await builder.build({outputDir, workspace, crate: 'snip20-reference-impl'})
    }),
    task('build mgmt', async () => {
      binaries.MGMT = await builder.build({outputDir, workspace, crate: 'sienna-mgmt'})
    }),
    task('build rpt', async () => {
      binaries.RPT = await builder.build({outputDir, workspace, crate: 'sienna-rpt'})
    })
  )
  return binaries
}

export async function upload ({
  task    = taskmaster(),
  builder = new SecretNetwork.Builder(),
  binaries
} = {}) {
  const receipts = {}
  await task('upload token', async () => {
    receipts.TOKEN = await builder.uploadCached(binaries.TOKEN)
    console.log(`⚖️  compressed size ${receipts.TOKEN.compressedSize} bytes`)
  })
  await task('upload mgmt', async () => {
    receipts.MGMT = await builder.uploadCached(binaries.MGMT)
    console.log(`⚖️  compressed size ${receipts.MGMT.compressedSize} bytes`)
  })
  await task('upload rpt', async () => {
    receipts.RPT = await builder.uploadCached(binaries.RPT)
    console.log(`⚖️  compressed size ${receipts.RPT.compressedSize} bytes`)
  })
  return receipts
}

export async function initialize ({
  task = taskmaster(),
  agent,
  receipts,
  inits
}) {
  const contracts = {}
  const initTXs = {}
  await task('initialize token', async report => {
    const {codeId} = receipts.TOKEN
    contracts.TOKEN = new SNIP20Contract({ agent, codeId })
    report(await contracts.TOKEN.init(inits.TOKEN))
  })
  await task('initialize mgmt', async report => {
    const {codeId} = receipts.MGMT
    inits.MGMT.initMsg.token = [contracts.TOKEN.address, contracts.TOKEN.codeHash]
    contracts.MGMT = new MGMTContract({ agent, codeId })
    report(await contracts.MGMT.init(inits.MGMT))
  })
  await task('initialize rpt', async report => {
    const {codeId} = receipts.RPT
    inits.RPT.initMsg.token = [contracts.TOKEN.address, contracts.TOKEN.codeHash]
    inits.RPT.initMsg.mgmt  = [contracts.MGMT.address, contracts.MGMT.codeHash]
    contracts.RPT = new RPTContract({ agent, codeId })
    report(await contracts.RPT.init(inits.RPT))
  })
  return contracts
}

export async function launch () {}

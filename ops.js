import { taskmaster } from '@hackbg/fadroma'
import { resolve, dirname, fileURLToPath } from '@hackbg/fadroma/js/sys.js'

export default async function deploy ({
  task = taskmaster(),
  builder,
  initMsgs
}) {
  return await task('build, upload, and initialize contracts', async () => {
    const binaries = await build({ task, builder })
    const receipts = await upload({ task, builder, binaries })
    const contracts = await initialize({ task, builder, initMsgs })
  })
}

export async function build ({
  task = taskmaster(),
  builder,
  workspace = resolve(dirname(fileURLToPath(import.meta.url)))
} = {}) {
  const binaries = {}
  await task.parallel(
    'build contracts',
    task('build token', async () => {
      binaries.TOKEN = await builder.build({workspace, crate: 'snip20-reference-impl'})
    }),
    task('build mgmt', async () => {
      binaries.MGMT = await builder.build({workspace, crate: 'snip20-reference-impl'})
    }),
    task('build rpt', async () => {
      binaries.RPT = await builder.build({workspace, crate: 'snip20-reference-impl'})
    })
  )
  return binaries
}

export async function upload ({
  task = taskmaster(),
  builder,
  binaries
} = {}) {
  const receipts = {}
  return await task.parallel(
    'upload contracts',
    ...Object.entries({
      TOKEN: 'upload token',
      MGMT:  'upload mgmt',
      RPT:   'upload rpt'
    }).map(([id, info]=>task(info, async () => {
      receipts[id] = await builder.uploadCached(binaries[id])
      console.info(`⚖️  compressed size ${receipts[id].compressedSize} bytes`)
    })
  )
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
    contracts.TOKEN = new SNIP20Contract({ agent, codeId: receipts.TOKEN.id })
    report(await contracts.TOKEN.init(inits.TOKEN))
  })
  await task('initialize mgmt', async () => {
    contracts.MGMT = new MGMTContract({ agent, codeId: receipts.MGMT.id })
    report(await contracts.MGMT.init(inits.MGMT))
  })
  await task('initialize rpt', async () => {
    contracts.RPT = new RPTContract({ agent, codeId: receipts.RPT.id })
    report(await contracts.RPT.init(inits.RPT))
  })
  return contracts
}

export async function launch () {}

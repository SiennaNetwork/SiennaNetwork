import YAML from 'js-yaml'
import alignYAML from 'align-yaml'
import {
  bold,
  resolve, dirname, rimraf,
  writeFileSync, bold, readdirSync, statSync, unlinkSync,
  Deployment
} from '@hackbg/fadroma'
import { workspace } from '@sienna/settings'

export async function fixReceipts ({ agent, deployment }) {

  for (const chainId of [
    'fadroma-scrt-12',
    'holodeck-2',
    'mocknet',
    'pulsar-1',
    'pulsar-2',
    'secret-4'
  ]) {
    const deployments = resolve(workspace, 'receipts', chainId, 'deployments')
    for (const prefix of readdirSync(deployments)) {
      if (prefix === '.active') continue
      const path = resolve(deployments, prefix)
      const stats = statSync(path)
      if (stats.isDirectory() && !stats.isSymbolicLink()) {
        const deployment = new Deployment(path)
        const yaml = toYAML(deployment)
        console.log(`${path}.yml`, yaml.length)
        writeFileSync(`${path}.yml`, yaml)
        rimraf(path)
      }
    }
  }

}

export function toYAML (deployment) {

  let output = ''

  for (let [name, data] of Object.entries(deployment.receipts)) {
    output += '---\n'
    if (data.initTx) {
      if (data.initTx.contractAddress) {
        data.address = data.initTx.contractAddress
      }
      if (data.initTx.transactionHash) {
        data.initTx = data.initTx.transactionHash
      } else {
        delete data.initTx
      }
    } else if (data.contractAddress) {
      data = {
        codeId:  Number(data.logs[0].events[0].attributes.filter(x=>x.key==='code_id')[0].value),
        address: data.contractAddress,
        initTx:  data.transactionHash,
      }
    }/* else if (data.exchange) {
      const pair = [data.token_0, data.token_1]
      data = {
        pair,
        address:  data.exchange.address,
        lp_token: data.lp_token,
      }
    }*/
    output += alignYAML(YAML.dump({ name, ...data }, { noRefs: true }))
  }

  return output

}

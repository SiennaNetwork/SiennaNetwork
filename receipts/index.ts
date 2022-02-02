import YAML from 'js-yaml'
import alignYAML from 'align-yaml'
import {
  bold,
  resolve, dirname, rimraf,
  readFileSync, writeFileSync, readdirSync, statSync, unlinkSync,
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
      console.log(chainId, prefix)
      if (prefix.endsWith('.yml')) {
        const path = resolve(deployments, prefix)
        let output = ''
        for (const contract of YAML.loadAll(readFileSync(path, 'utf8'))) {
          if (contract.name === 'SiennaSNIP20') {
            contract.name = 'SIENNA'
          }
          if (contract.name.startsWith('Sienna')) {
            contract.name = contract.name.slice('Sienna'.length)
          }
          if (contract.name.startsWith('Rewards_')) {
            contract.name = 'Rewards[v2].' + contract.name.slice(
              'Rewards_'.length,
              contract.name.length-'_Pool'.length
            )
          }
          if (contract.name.endsWith('_LP')) {
            contract.name = contract.name.slice(0,contract.name.length-'_LP'.length)+'.LP'
          }
          if (contract.name.startsWith('Swap_')) {
            contract.name = 'AMM[v1].' + contract.name.slice('Swap_'.length)
          }
          if (contract.name === 'AMMFactory') {
            contract.name = 'AMM[v1].Factory'
          }
          if (contract.name === 'AMMFactory@v1') {
            contract.name = 'AMM[v1].Factory'
          }
          if (contract.name === 'AMMFactory@v2') {
            contract.name = 'AMM[v2].Factory'
          }
          output += '---\n'
          output += alignYAML(YAML.dump(contract, { noRefs: true }))
        }
        writeFileSync(`${path}`, output)
      }
      //if (prefix === '.active') continue
      //const path = resolve(deployments, prefix)
      //const stats = statSync(path)
      //if (stats.isDirectory() && !stats.isSymbolicLink()) {
        //const deployment = new Deployment(path)
        //const yaml = toYAML(deployment)
        //console.log(`${path}.yml`, yaml.length)
        //writeFileSync(`${path}.yml`, yaml)
        //rimraf(path)
      //}
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

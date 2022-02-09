import YAML from 'js-yaml'
import alignYAML from 'align-yaml'
import {
  bold,
  resolve, dirname, rimraf,
  readFileSync, writeFileSync, readdirSync, statSync, unlinkSync,
  Deployment,
  Console
} from '@hackbg/fadroma'
import { workspace } from '@sienna/settings'

const console = Console('@sienna/receipts')

export async function fix1 ({ agent, deployment }) {

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

export async function fix2 ({ agent, deployment }) {
  const source = resolve(workspace, 'receipts/secret-4/deployments/prod.yml')
  const input  = YAML.loadAll(readFileSync(source, 'utf8'))
  const target = resolve(workspace, 'receipts/secret-4/deployments/prod.fixed.yml')
  const output = []
  for (const instance of input) {
    if (instance.exchange) {
      const name = instance.name
      delete instance.name
      console.info('Querying code hash for', instance.exchange.address)
      const codeHash = await agent.getCodeHash(instance.exchange.address)
      output.push({name, address: instance.exchange.address, codeHash, ...instance})
      output.push({name: `${name}.LP`, address: instance.lp_token.address, codeHash: instance.lp_token.code_hash,...instance, })
      continue
    }
    if (instance.name.startsWith('Rewards[v2]')) {
      const [_,lp] = instance.name.split('.')
      if (lp === 'SIENNA') {
        instance.name = 'SIENNA.Rewards[v2]'
      } else {
        instance.name = `AMM[v1].${lp}.LP.Rewards[v2]`
      }
    }
    output.push(instance)
  }
  const result = alignYAML(
    output.map(receipt=>YAML.dump(receipt)).join('---\n')
  )
  writeFileSync(target, result, 'utf8')
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

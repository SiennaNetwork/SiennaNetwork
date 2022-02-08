import { Console, bold } from '@hackbg/fadroma'

const console = Console('@sienna/snip20-sienna')

import { Snip20Contract_1_0, Source } from '@hackbg/fadroma'
import { workspace } from '@sienna/settings'
import { SiennaSnip20Client } from './SiennaSnip20Client'
export { SiennaSnip20Client }
export class SiennaSnip20Contract extends Snip20Contract_1_0 {
  name   = 'SIENNA'
  source = { workspace, crate: 'snip20-sienna' }
  Client = SiennaSnip20Client

  /* Command. Print balance of active agent in SIENNA token. */
  static status = siennaStatus

  /* Command. Send some SIENNA to predefined addresses. */
  static fundTesters = fundTesters
}

async function siennaStatus ({ deployment, agent, cmdArgs }) {
  console.log({agent})
  const [ vk = 'q1Y3S7Vq8tjdWXCL9dkh' ] = cmdArgs
  const sienna = new SiennaSnip20Client({ ...deployment.get('SIENNA'), agent })
  console.log({sienna})
  try {
    const balance = await sienna.getBalance(agent.address, vk)
    console.info(`SIENNA balance of ${bold(agent.address)}: ${balance}`)
  } catch (e) {
    if (agent.chain.isMainnet) {
      throw new Error('SIENNA mainnet: pass real vk')
    }
    const VK = await sienna.setViewingKey(vk)
    console.log(VK)
    console.error(e.message)
  }
}

async function fundTesters ({ deployment, agent, cmdArgs }) {
  const [ vk = 'q1Y3S7Vq8tjdWXCL9dkh' ] = cmdArgs
  const sienna  = new SiennaSnip20Client({ ...deployment.get('SIENNA'), agent })
  const balanceBefore = await sienna.getBalance(agent.address, vk)
  console.info(`SIENNA balance of ${bold(agent.address)}: ${balanceBefore}`)
  const amount  = balanceBefore.slice(0, balanceBefore.length - 1)
  await sienna.transfer(amount, 'secret13nkfwfp8y9n226l9sy0dfs0sls8dy8f0zquz0y')
  await sienna.transfer(amount, 'secret1xcywp5smmmdxudc7xgnrezt6fnzzvmxqf7ldty')
  const balanceAfter = await sienna.getBalance(agent.address, vk)
  console.info(`SIENNA balance of ${bold(agent.address)}: ${balanceAfter}`)
}

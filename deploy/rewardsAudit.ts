import { SiennaSNIP20, LPToken, RewardsContract } from '@sienna/api'
import { bold, timestamp, Console } from '@fadroma/tools'

import init from './init'
import buildAndUpload from './buildAndUpload'

const console = Console(import.meta.url)

export default {

  async ['deploy'] () {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
    const prefix  = `AUDIT-${timestamp()}`
    const SIENNA  = new SiennaSNIP20({ prefix, admin })
    const LPTOKEN = new LPToken({ prefix, admin, name: 'AUDIT' })
    const REWARDS = new RewardsContract({
      prefix, admin, name: 'AUDIT',
      lpToken: LPTOKEN, rewardToken: SIENNA
    })
    await buildAndUpload([SIENNA, LPTOKEN, REWARDS])
    await SIENNA.instantiate()
    await LPTOKEN.instantiate()
    await REWARDS.instantiate()
    await SIENNA.setMinters([admin.address])
    await chain.instances.select(prefix)
    console.debug(`Deployed the following contracts to ${bold(chain.chainId)}:`, {
      SIENNA:  SIENNA.link,
      LPTOKEN: LPTOKEN.link,
      REWARDS: REWARDS.link
    })
  },

  async ['epoch'] (amount: string|number) {
    amount = Number(amount)
    if (isNaN(amount) || amount < 0) {
      throw new Error('pass a non-negative amount of rewards to vest for this epoch')
    }

    const {chain, admin} = await init(process.env.CHAIN_NAME)
    const instance = chain.instances.active
    const SIENNA   = instance.getContract(SiennaSNIP20, 'SiennaSNIP20', admin)
    const REWARDS  = instance.getContract(RewardsContract, 'SiennaRewards_AUDIT_Pool', admin)

    await SIENNA.tx.mint({
      amount:    String(amount),
      recipient: REWARDS.address,
      padding:   null
    }, admin)

    const info = await REWARDS.Q(admin).poolInfo()
    console.debug(info)

    await REWARDS.TX(admin).beginEpoch(0)
  },

  async ['deposit'] () {
    const {chain} = await init(process.env.CHAIN_NAME)
  },

  async ['withdraw'] () {
    const {chain} = await init(process.env.CHAIN_NAME)
  },

  async ['claim'] () {
    const {chain} = await init(process.env.CHAIN_NAME)
  },

  async ['enable-migration'] () {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
  },

  async ['migrate'] () {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
  },

}

import { bold, timestamp, Console } from '@hackbg/fadroma'

const console = Console('[@sienna/amm/rewardsAudit]'])

import {
  SiennaSNIP20Contract,
  LPTokenContract,
  RewardsContract
} from '@sienna/api'

export const rewardsAudit = {

  async ['deploy'] ({ chain, admin, args: [ bonding ] }) {
    bonding = Number(bonding)
    if (isNaN(bonding) || bonding < 0) {
      throw new Error('pass a non-negative bonding period to configure (in seconds)')
    }
    const prefix  = `AUDIT-${timestamp()}`
    const SIENNA  = new SiennaSNIP20Contract({ prefix, admin })
    const LPTOKEN = new LPTokenContract({ prefix, admin, name: 'AUDIT' })
    const REWARDS = new RewardsContract({
      prefix, admin, name: 'AUDIT',
      lpToken: LPTOKEN, rewardToken: SIENNA
    })
    await chain.buildAndUpload([SIENNA, LPTOKEN, REWARDS])
    await SIENNA.instantiate()
    await LPTOKEN.instantiate()
    await REWARDS.instantiate()
    await SIENNA.tx().setMinters([admin.address])
    await chain.deployments.select(prefix)
    console.debug(`Deployed the following contracts to ${bold(chain.chainId)}:`, {
      SIENNA:  SIENNA.link,
      LPTOKEN: LPTOKEN.link,
      REWARDS: REWARDS.link
    })
  },

  async ['epoch'] ({ chain, admin, args: [amount] }) {
    amount = Number(amount)
    if (isNaN(amount) || amount < 0) {
      throw new Error('pass a non-negative amount of rewards to vest for this epoch')
    }
    amount = String(amount)

    const instance = chain.deployments.active
    const SIENNA   = instance.getContract(SiennaSNIP20Contract, 'SiennaSNIP20', admin)
    const REWARDS  = instance.getContract(RewardsContract, 'SiennaRewards_AUDIT_Pool', admin)

    await SIENNA.tx(admin).mint(amount, REWARDS.address)

    const epoch = (await REWARDS.epoch) + 1
    await REWARDS.tx(admin).beginEpoch(epoch)

    console.info(`Started epoch ${bold(String(epoch))} with reward budget: ${bold(amount)}`)
  },

  async ['status'] ({ chain, admin, args: [string] }) {
    const instance = chain.deployments.active
    const REWARDS  = instance.getContract(RewardsContract, 'SiennaRewards_AUDIT_Pool', admin)
    if (identity) {
      const {address} = chain.identities.load(identity)
      console.debug('User info:', await REWARDS.q(admin).user_info(address))
    } else {
      console.debug('Pool info:', await REWARDS.q(admin).pool_info())
    }
  },

  async ['deposit'] ({ chain, admin, args: [ user, amount ] }) {
    if (!user) {
      chain.printIdentities()
      throw new Error('pass an identity to deposit')
    }
    amount = Number(amount)
    if (isNaN(amount) || amount < 0) {
      throw new Error('pass a non-negative amount of LP tokens to deposit')
    }
    amount = String(amount)
    const {mnemonic} = chain.identities.load(user)
    const agent    = await chain.getAgent({mnemonic})
    const instance = chain.deployments.active
    const REWARDS  = instance.getContract(RewardsContract, 'SiennaRewards_AUDIT_Pool', admin)
    const LPTOKEN  = instance.getContract(LPTokenContract, 'SiennaRewards_AUDIT_LPToken', admin)

    await LPTOKEN.tx(admin).mint(amount, agent.address)
    await LPTOKEN.tx(admin).increaseAllowance(amount, REWARDS.address)
    await REWARDS.tx(agent).deposit(amount)

    console.info(`Deposited ${bold(amount)} LPTOKEN from ${bold(agent.address)} (${user})`)
  },

  async ['withdraw'] ({ chain, admin, args: [ user, amount ] }) {
    if (!user) {
      chain.printIdentities()
      throw new Error('pass an identity to withdraw')
    }
    amount = Number(amount)
    if (isNaN(amount) || amount < 0) {
      throw new Error('pass a non-negative amount of LP tokens to withdraw')
    }
    amount = String(amount)
    const {mnemonic} = chain.identities.load(user)
    const agent    = await chain.getAgent({mnemonic})
    const instance = chain.deployments.active
    const REWARDS  = instance.getContract(RewardsContract, 'SiennaRewards_AUDIT_Pool', admin)

    await REWARDS.tx(agent).withdraw(amount)

    console.info(`Withdrew ${bold(amount)} LPTOKEN from ${bold(agent.address)} (${user})`)
  },

  async ['claim'] ({ chain, admin, args: [ user ]}) {
    if (!user) {
      chain.printIdentities()
      throw new Error('pass an identity to claim')
    }
    const {mnemonic} = chain.identities.load(user)
    const agent    = await chain.getAgent({mnemonic})
    const instance = chain.deployments.active
    const REWARDS  = instance.getContract(RewardsContract, 'SiennaRewards_AUDIT_Pool', admin)

    await REWARDS.tx(agent).claim()

    console.info(`Claimed`)
  },

  async ['enable-migration'] () {
  },

  async ['migrate'] () {
  },

}

import { bold, timestamp, Console } from '@hackbg/tools'

const console = Console(import.meta.url)

import {
  SiennaSNIP20Contract,
  LPTokenContract,
  RewardsContract
} from '@sienna/api'

import { init, buildAndUpload } from '@fadroma/scrt'

export const rewardsAudit = {

  async ['deploy'] (bonding: number) {
    bonding = Number(bonding)
    if (isNaN(bonding) || bonding < 0) {
      throw new Error('pass a non-negative bonding period to configure (in seconds)')
    }
    const {chain, admin} = await init(process.env.CHAIN_NAME)
    const prefix  = `AUDIT-${timestamp()}`
    const SIENNA  = new SiennaSNIP20Contract({ prefix, admin })
    const LPTOKEN = new LPTokenContract({ prefix, admin, name: 'AUDIT' })
    const REWARDS = new RewardsContract({
      prefix, admin, name: 'AUDIT',
      lpToken: LPTOKEN, rewardToken: SIENNA
    })
    await buildAndUpload([SIENNA, LPTOKEN, REWARDS])
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

  async ['epoch'] (amount: string|number) {
    amount = Number(amount)
    if (isNaN(amount) || amount < 0) {
      throw new Error('pass a non-negative amount of rewards to vest for this epoch')
    }
    amount = String(amount)

    const {chain, admin} = await init(process.env.CHAIN_NAME)
    const instance = chain.deployments.active
    const SIENNA   = instance.getContract(SiennaSNIP20Contract, 'SiennaSNIP20', admin)
    const REWARDS  = instance.getContract(RewardsContract, 'SiennaRewards_AUDIT_Pool', admin)

    await SIENNA.tx(admin).mint(amount, REWARDS.address)

    const epoch = (await REWARDS.epoch) + 1
    await REWARDS.tx(admin).beginEpoch(epoch)

    console.info(`Started epoch ${bold(String(epoch))} with reward budget: ${bold(amount)}`)
  },

  async ['status'] (identity: string) {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
    const instance = chain.deployments.active
    const REWARDS  = instance.getContract(RewardsContract, 'SiennaRewards_AUDIT_Pool', admin)
    if (identity) {
      const {address} = chain.identities.load(identity)
      console.debug('User info:', await REWARDS.q(admin).user_info(address))
    } else {
      console.debug('Pool info:', await REWARDS.q(admin).pool_info())
    }
  },

  async ['deposit'] (user: string, amount: string|number) {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
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

  async ['withdraw'] (user: string, amount: string|number) {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
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

  async ['claim'] (user: string) {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
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
    const {chain, admin} = await init(process.env.CHAIN_NAME)
  },

  async ['migrate'] () {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
  },

}

import {
  Scrt_1_2, ContractConstructor,
  printContract, printContracts,
  MigrationContext, randomHex, timestamp,
  bold, Console,
} from "@hackbg/fadroma"
const console = Console('@sienna/rewards')
import { SNIP20Contract } from '@fadroma/snip20'
import getSettings, { workspace, SIENNA_DECIMALS, ONE_SIENNA } from '@sienna/settings'
import {
  SiennaSNIP20Contract,
  FactoryContract, AMMContract,
  ExchangeInfo, LPTokenContract,
  RPTContract, RPTConfig,
} from '@sienna/api'

import { Init } from './schema/init.d'
export * from './RewardsApi'
import { RewardsTransactions, RewardsQueries, RewardsAPIVersion } from './RewardsApi'

export abstract class RewardsContract extends Scrt_1_2.Contract<RewardsTransactions, RewardsQueries> {

  name  = this.name  || 'SiennaRewards'
  crate = this.crate || 'sienna-rewards'
  abstract version: RewardsAPIVersion

  RewardTokenContract: ContractConstructor<SNIP20Contract> = SiennaSNIP20Contract
  abstract rewardToken <T extends SNIP20Contract> (
    Contract: ContractConstructor<SNIP20Contract>
  ): Promise<T>

  LPTokenContract: ContractConstructor<SNIP20Contract> = LPTokenContract
  abstract lpToken <T extends SNIP20Contract> (
    Contract: ContractConstructor<SNIP20Contract>
  ): Promise<T>

  static v2 = class RewardsContract_v2 extends RewardsContract {
    ref     = 'rewards-2.1.2'
    version = 'v2' as RewardsAPIVersion
    initMsg?: any // TODO v2 init type
    Transactions = RewardsTransactions // TODO v2 executors
    Queries      = RewardsQueries
    constructor (input) {
      super(input)
      const { lpToken, rewardToken, admin } = input
      this.initMsg = {
        admin:        admin?.address,
        lp_token:     lpToken?.link,
        reward_token: rewardToken?.link,
        viewing_key:  "",
        ratio:        ["1", "1"],
        threshold:    15940,
        cooldown:     15940,
      }
    }
    async lpToken <T extends SNIP20Contract> (T = this.LPTokenContract): Promise<T> {
      const at = Math.floor(+new Date()/1000)
      const {pool_info} = await this.query({pool_info:{at}})
      const {address, code_hash} = pool_info.lp_token
      return new T({ address, codeHash: code_hash, agent: this.agent }) as T
    }
    async rewardToken <T extends SNIP20Contract> (T = this.LPTokenContract): Promise<T> {
      throw new Error('not implemented')
    }
  }

  static v3 = class RewardsContract_v3 extends RewardsContract {
    version = 'v3' as RewardsAPIVersion
    initMsg?: Init
    Transactions = RewardsTransactions
    Queries      = RewardsQueries
    constructor (input) {
      super(input)
      const { lpToken, rewardToken, admin } = input
      this.initMsg = {
        admin: admin?.address,
        config: {
          reward_vk:    randomHex(36),
          bonding:      86400,
          timekeeper:   admin?.address,
          lp_token:     lpToken?.link,
          reward_token: rewardToken?.link,
        }
      }
    }
    async lpToken (SNIP20 = this.LPTokenContract): Promise<any> {
      throw new Error('v3 does not expose config')
    }
    async rewardToken (SNIP20 = this.RewardTokenContract): Promise<any> {
      throw new Error('v3 does not expose config')
    }
    get epoch (): Promise<number> {
      return this.q().pool_info().then(pool_info=>pool_info.clock.number)
    }
  }

}

export async function deployRewards ({
  deployment, admin, run,
  SIENNA  = deployment.getThe('SiennaSNIP20', new SiennaSNIP20Contract({ admin })),
  version = 'v3'
}) {
  const { SSSSS_POOL, RPT_CONFIG_SSSSS } =
    await run(deploySSSSS, { SIENNA, version })
  const { REWARD_POOLS, RPT_CONFIG_SWAP_REWARDS } =
    await run(deployRewardPools, { SIENNA, version })
  return {
    REWARD_POOLS: [ SSSSS_POOL, ...REWARD_POOLS ],
    RPT_CONFIG:   [ ...RPT_CONFIG_SSSSS, ...RPT_CONFIG_SWAP_REWARDS ]
  }
}
Object.assign(deployRewards, {

  /** Deploy legacy Rewards v2. */
  v2: async function deployRewards_v2 ({run}) {
    const { RPT_CONFIG, REWARD_POOLS } = await run(deployRewards, { version: 'v2' })
    return await run(adjustRPTConfig, { RPT_CONFIG })
  },

  /** Deploy latest Rewards v3. */
  v3: async function deployRewards_v3 ({run}) {
    const { RPT_CONFIG, REWARD_POOLS } = await run(deployRewards, { version: 'v3' })
    return await run(adjustRPTConfig, { RPT_CONFIG })
  },

  /** Deploy both versions simultaneously,
    * splitting the balance evenly in the RPT config. */
  v2_and_v3: async function deployRewards_v2_and_v3 ({run}) {
    const [V2, V3] = await Promise.all([
      run(deployRewards, { version: 'v2' }),
      run(deployRewards, { version: 'v3' })
    ])
    const REWARD_POOLS = [ ...V2.REWARD_POOLS, ...V3.REWARD_POOLS ]
    console.table(REWARD_POOLS.reduce(
      (table, {label, address, codeId, codeHash})=>
        Object.assign(table, {
          [label]: { address: address, codeId: codeId, codeHash: codeHash }
        }), {}))
    const RPT_CONFIG  = [ ...V2.RPT_CONFIG,   ...V3.RPT_CONFIG   ]
    return await run(adjustRPTConfig, { RPT_CONFIG })
    return { RPT_CONFIG, REWARD_POOLS }
  }

})

/** Deploy SIENNA/SIENNA SINGLE-SIDED STAKING,
  * (5- or 6-S depending on whether you count the SLASH)
  * a Sienna Rewards pool where you stake SIENNA to earn SIENNA. */
export async function deploySSSSS ({
  run, chain, deployment,
  SIENNA, version,
}) {
  const { REWARDS: SSSSS_POOL } = await run(deployRewardPool, {
    version,
    name:        'SSSSS',
    lpToken:     SIENNA,
    rewardToken: SIENNA,
  })
  return {
    SSSSS_POOL, RPT_CONFIG_SSSSS: [
      [
        SSSSS_POOL.address,
        String(BigInt(getSettings(chain.chainId).rewardPairs.SIENNA) * ONE_SIENNA)
      ]
    ]
  }
}

export async function deployRewardPool ({
  admin, chain, deployment, prefix,
  lpToken,
  rewardToken = deployment.getThe('SiennaSNIP20', new SiennaSNIP20Contract({ admin })),
  name        = 'UNKNOWN',
  version     = 'v3',
  suffix      = `+${timestamp()}`,
}) {
  name = `SiennaRewards_${version}_${name}`
  console.info(bold(`Deploying ${name}:`), version)
  console.info(bold(`Staked token:`), lpToken.address, lpToken.codeHash)
  const REWARDS = new RewardsContract[version]({
    workspace, name, suffix,
    lpToken, rewardToken, admin
  })
  await chain.buildAndUpload(admin, [REWARDS])
  await deployment.getOrCreateContract(admin, REWARDS, REWARDS.label, REWARDS.initMsg)
  return { REWARDS }
}

/** Deploy the rest of the reward pools,
  * where you stake a LP token to earn SIENNA. */
export async function deployRewardPools ({
  chain, admin, deployment, prefix, run,
  SIENNA  = deployment.getThe('SiennaSNIP20', new SiennaSNIP20Contract({ admin })),
  version = 'v3',
  ammVersion = 'v1',
  suffix  = `+${timestamp()}`,
  split   = 1.0,
}) {

  const REWARDS = new RewardsContract[version]({ workspace, prefix, admin })
  await chain.buildAndUpload(admin, [REWARDS])
  const REWARD_POOLS            = []
  const RPT_CONFIG_SWAP_REWARDS = []

  const { swapPairs } = getSettings(chain.chainId)
  if (!swapPairs || swapPairs.length === 1) {
    throw new Error('@sienna/rewards: needs swapPairs setting')
  }

  const { rewardPairs } = getSettings(chain.chainId)
  if (!rewardPairs || rewardPairs.length === 1) {
    throw new Error('@sienna/rewards: needs rewardPairs setting')
  }

  for (const name of swapPairs) {

    console.info(bold('Checking if rewards are allocated for'), name)
    if (!rewardPairs[name]) {
      console.info(bold('No rewards for'), name)
      continue
    }

    const exchangeName = `SiennaSwap_v1_${name}`
    console.info(bold('Need LP token of exchange'), exchangeName)

    const exchange = deployment.receipts[exchangeName]
    if (!exchange) {
      console.error(bold(`Exchange does not exist in deployment`), exchangeName)
      console.error(bold(`Contracts in deployment:`), Object.keys(deployment.receipts).join(' '))
      process.exit(1)
    }

    const lpToken = new LPTokenContract({
      address:  exchange.lp_token.address,
      codeHash: exchange.lp_token.code_hash,
      admin
    })
    console.info(bold('Found LP token:'), lpToken.address)

    const { REWARDS } = await run(deployRewardPool, {
      version, name, suffix,
      lpToken, rewardToken: SIENNA,
    })

    REWARD_POOLS.push(REWARDS)

    const reward = BigInt(rewardPairs[name]) / BigInt(1 / split)

    RPT_CONFIG_SWAP_REWARDS.push(
      [REWARDS.address, String(reward * ONE_SIENNA)]
    )

  }

  return { REWARD_POOLS, RPT_CONFIG_SWAP_REWARDS }
}

/** After deploying the SSSSS and the other reward pools,
  * set their addresses in the deployment's RPT contract. */
export async function adjustRPTConfig ({
  deployment, chain, admin,
  RPT = deployment.getThe('SiennaRPT', new RPTContract({ admin })),
  RPT_CONFIG,
}) {
  // on mainnet we use a multisig
  // so we can't run the transaction from here
  if (chain.isMainnet) {
    deployment.save({config: RPT_CONFIG}, 'RPTConfig.json')
    console.info(
      `\n\nWrote RPT config to deployment ${deployment.prefix}. `+
      `You should use this file as the basis of a multisig transaction.`
    )
    return
  }
  console.info(
    bold(`Configuring RPT`), RPT.address
  )
  for (const [address, amount] of RPT_CONFIG) {
    console.info(`- ${address} ${amount}`)
  }
  await RPT.tx(admin).configure(RPT_CONFIG)
  return { RPT_CONFIG }
}

export async function upgradeRewards ({
  timestamp, chain, admin, deployment, prefix, run,

  OldRewardsContract,
  NewRewardsContract,

  SIENNA = deployment.getThe('SiennaSNIP20', new SiennaSNIP20Contract({ admin })),
  RPT    = deployment.getThe('SiennaRPT',    new SiennaSNIP20Contract({ admin })),
  REWARD_POOLS = deployment.getAll(
    'SiennaRewards_v2',
    name => new OldRewardsContract({ agent: admin })
  ),

  version,
  suffix = `+${timestamp}`

}) {
  const NEW_REWARD_POOLS: RewardsContract[] = []
  for (const REWARDS of REWARD_POOLS) {
    const LP_TOKEN = await REWARDS.lpToken()
    const {symbol} = await LP_TOKEN.info
    let name
    if (symbol === 'SIENNA') {
      name = 'SSSSS'
    } else {
      const [LP, TOKEN0, TOKEN1] = (await LP_TOKEN.friendlyName).split('-')
      name = `${TOKEN0}-${TOKEN1}`
    }
    console.log()
    console.info(bold('Upgrading reward pool'), name)
    NEW_REWARD_POOLS.push((await run(deployRewardPool, {
      version, name, suffix,
      lpToken: LP_TOKEN, rewardToken: SIENNA
    })).REWARDS)
  }
  console.info(`Deployed`, bold(String(NEW_REWARD_POOLS.length)), version, `reward pools.`)
  return { REWARD_POOLS: NEW_REWARD_POOLS }
}
Object.assign(upgradeRewards, {
  v2_to_v3: function upgradeRewards_v2_to_v3 (input) {
    return upgradeRewards({
      ...input,
      OldRewardsContract: RewardsContract.v2,
      NewRewardsContract: RewardsContract.v3,
      version: 'v3'
    })
  }
})

type MultisigTX = any
const pick       = (...keys) => x => keys.reduce((y, key)=>{y[key]=x[key];return y}, {})
const essentials = pick('codeId', 'codeHash', 'address', 'label')
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
    const deployment = chain.deployments.active
    const SIENNA   = deployment.getContract(SiennaSNIP20Contract, 'SiennaSNIP20', admin)
    const REWARDS  = deployment.getContract(RewardsContract, 'SiennaRewards_AUDIT_Pool', admin)
    await SIENNA.tx(admin).mint(amount, REWARDS.address)
    const epoch = (await REWARDS.epoch) + 1
    await REWARDS.tx(admin).beginEpoch(epoch)
    console.info(`Started epoch ${bold(String(epoch))} with reward budget: ${bold(amount)}`)
  },
  async ['status'] ({ chain, admin, args: [string] }) {
    const deployment = chain.deployments.active
    const REWARDS  = deployment.getContract(RewardsContract, 'SiennaRewards_AUDIT_Pool', admin)
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
    const deployment = chain.deployments.active
    const REWARDS  = deployment.getContract(RewardsContract, 'SiennaRewards_AUDIT_Pool', admin)
    const LPTOKEN  = deployment.getContract(LPTokenContract, 'SiennaRewards_AUDIT_LPToken', admin)
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
    const deployment = chain.deployments.active
    const REWARDS  = deployment.getContract(RewardsContract, 'SiennaRewards_AUDIT_Pool', admin)
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
    const deployment = chain.deployments.active
    const REWARDS  = deployment.getContract(RewardsContract, 'SiennaRewards_AUDIT_Pool', admin)
    await REWARDS.tx(agent).claim()
    console.info(`Claimed`)
  },
  async ['enable-migration'] () {
  },
  async ['migrate'] () {
  },
}

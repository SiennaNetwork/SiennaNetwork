import { QueryExecutor, ContractConstructor } from '@fadroma/scrt'

export class RewardsQueries extends QueryExecutor {

  async poolInfo (at = Math.floor(+ new Date() / 1000)) {
    const result = await this.query({ rewards: { pool_info: { at } } })
    return result.rewards.pool_info
  }

  async getEpoch () {
    const info = await this.poolInfo()
    return info.clock.number
  }

  async getRewardToken (TOKEN: ContractConstructor) {
    const { address, code_hash } = (await this.poolInfo()).reward_token
    return new TOKEN({ address, codeHash: code_hash, admin: this.agent })
  }

  async getLPToken (TOKEN: ContractConstructor) {
    const { address, code_hash } = (await this.poolInfo()).lp_token
    return new TOKEN({ address, codeHash: code_hash, admin: this.agent })
  }

}

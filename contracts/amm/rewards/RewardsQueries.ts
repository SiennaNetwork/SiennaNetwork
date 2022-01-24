import { QueryExecutor, ContractConstructor } from '@hackbg/fadroma'

export class RewardsQueries extends QueryExecutor {

  async pool_info (at = Math.floor(+ new Date() / 1000)) {
    const result = await this.query({ rewards: { pool_info: { at } } })
    return result.rewards.pool_info
  }

}

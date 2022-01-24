import { QueryExecutor } from '@hackbg/fadroma'

export class AMMQueries extends QueryExecutor {
  async pair_info () {
    const { pair_info } = await this.query("pair_info")
    return pair_info
  }
}

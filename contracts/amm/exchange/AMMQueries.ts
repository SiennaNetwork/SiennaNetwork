import { QueryExecutor } from '@hackbg/fadroma'

export class AMMQueries extends QueryExecutor {
  pair_info () {
    return this.query("pair_info")
  }
}

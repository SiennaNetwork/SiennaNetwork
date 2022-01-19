import { QueryExecutor } from '@fadroma/scrt'

export class AMMQueries extends QueryExecutor {
  pair_info () {
    return this.query("pair_info")
  }
}

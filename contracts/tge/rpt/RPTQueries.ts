import { QueryExecutor } from '@hackbg/fadroma'

export class RPTQueries extends QueryExecutor {

  async status () {
    const msg = { status: {} }
    return this.query(msg)
  }

}

import { QueryExecutor } from '@fadroma/scrt'

export class RPTQueries extends QueryExecutor {

  async status () {
    const msg = { status: {} }
    return this.agent.query(this.contract, msg)
  }

}

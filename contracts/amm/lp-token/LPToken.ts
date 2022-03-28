import { Snip20Client } from '@hackbg/fadroma'

export class LPTokenClient extends Snip20Client {

  async getPairName (): Promise<string> {
    const { agent } = this
    const { chain } = agent
    const { name } = await this.getTokenInfo()
    const fragments = name.split(' ')
    const [t0addr, t1addr] = fragments[fragments.length-1].split('-')
    const t0 = new Snip20Client({ agent, address: t0addr })
    const t1 = new Snip20Client({ agent, address: t1addr })
    const [t0info, t1info] = await Promise.all([t0.getTokenInfo(), t1.getTokenInfo()])
    return `${t0info.symbol}-${t1info.symbol}`
  }

}

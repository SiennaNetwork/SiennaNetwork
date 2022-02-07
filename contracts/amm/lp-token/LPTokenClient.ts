import { Snip20Client } from '@hackbg/fadroma'

export class LPTokenClient extends Snip20Client {
  getFriendlyName (): Promise<string> {
    const { agent } = this
    const { chain } = agent
    return this.info.then(async ({name})=>{
      const fragments = name.split(' ')
      const [t0addr, t1addr] = fragments[fragments.length-1].split('-')
      const t0 = new Snip20Contract_1_2({ chain, agent, address: t0addr })
      const t1 = new Snip20Contract_1_2({ chain, agent, address: t1addr })
      const [t0info, t1info] = await Promise.all([t0.info, t1.info])
      return `LP-${t0info.symbol}-${t1info.symbol}`
    })
  }
}

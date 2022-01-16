import type { IChain, IAgent } from '@fadroma/ops'
import { init } from '@fadroma/ops'
import { CHAINS } from '@fadroma/scrt'
export default async function (
  chainName: string,
): Promise<{
  chain: IChain,
  admin: IAgent
}> {
  return init(CHAINS, chainName)
}

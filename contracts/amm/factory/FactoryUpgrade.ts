import {
  Console, bold, timestamp, randomHex, printContract
} from '@hackbg/fadroma'

const console = Console('@sienna/factory/Upgrade')

import { ExchangeInfo, saveExchange, printExchanges } from '@sienna/exchange'

import { AMMFactoryContract } from './FactoryContract'
import { deployAMMFactory } from './FactoryDeploy'

export const upgradeAMM = {

  async v1_to_v2 ({
    run, chain, agent, deployment, prefix,
    FACTORY = deployment.getThe('AMM[v1].Factory', new AMMFactoryContract({agent, version: 'v1'})),
  }) {
    console.log()
    console.info(bold('Current factory:'))
    printContract(FACTORY)
    const EXCHANGES: ExchangeInfo[] = await FACTORY.exchanges
    await printExchanges(EXCHANGES)
    const version = 'v2'
    const { FACTORY: NEW_FACTORY } = await run(deployAMMFactory, { version, copyFrom: FACTORY })
    printContract(NEW_FACTORY)
    const NEW_EXCHANGES = []
    if (!EXCHANGES) {
      console.warn('No exchanges in old factory.')
    } else {
      for (const { name, TOKEN_0, TOKEN_1 } of EXCHANGES) {
        console.log()
        console.info(bold('Creating V2 exchange'), name, 'from corresponding V1 exchange')
        NEW_EXCHANGES.push(saveExchange(
          { deployment, version },
          await NEW_FACTORY.getContracts(),
          await NEW_FACTORY.createExchange(TOKEN_0, TOKEN_1)))
      }
      await printExchanges(NEW_EXCHANGES)
    }
    return { FACTORY: NEW_FACTORY, EXCHANGES: NEW_EXCHANGES }
  }
}

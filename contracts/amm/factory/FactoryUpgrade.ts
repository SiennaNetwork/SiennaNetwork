import {
  Console, bold, timestamp, randomHex, printContract
} from '@hackbg/fadroma'

const console = Console('@sienna/factory/Upgrade')

import { ExchangeInfo, saveExchange, printExchanges } from '@sienna/exchange'

import { AMMFactoryContract } from './FactoryContract'
import { deployAMMFactory } from './FactoryDeploy'

export async function upgradeAMM ({
  run, chain, agent, deployment, prefix,
  oldVersion = 'v1',
  FACTORY = deployment.getThe(
    `AMM[${oldVersion}].Factory`, 
    new AMMFactoryContract({agent, version: oldVersion})
  ),
  newVersion = 'v2',
}) {
  console.log()
  console.info(bold('Current factory:'))
  printContract(FACTORY)
  const EXCHANGES: ExchangeInfo[] = await FACTORY.exchanges
  await printExchanges(EXCHANGES)
  const { FACTORY: NEW_FACTORY } = await run(deployAMMFactory, { version: newVersion, copyFrom: FACTORY })
  printContract(NEW_FACTORY)
  const NEW_EXCHANGES = []
  if (!EXCHANGES) {
    console.warn('No exchanges in old factory.')
  } else {
    for (const { name, TOKEN_0, TOKEN_1 } of EXCHANGES) {
      console.log()
      console.info(bold('Creating V2 exchange'), name, 'from corresponding V1 exchange')
      NEW_EXCHANGES.push(saveExchange(
        { deployment, version: newVersion },
        await NEW_FACTORY.getContracts(),
        await NEW_FACTORY.createExchange(TOKEN_0, TOKEN_1)))
    }
    await printExchanges(NEW_EXCHANGES)
  }
  return { FACTORY: NEW_FACTORY, EXCHANGES: NEW_EXCHANGES }
}

Object.assign(upgradeAMM, {
  v1_to_v2 (input) { return upgradeAMM({ ...input, oldVersion: 'v1', newVersion: 'v2' }) }
})

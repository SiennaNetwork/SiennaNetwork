import { Console, bold, timestamp, randomHex, printContract, printContracts } from '@hackbg/fadroma'
const console = Console('@sienna/factory/Deploy')

import getSettings, { workspace } from '@sienna/settings'

import { AMMExchangeContract, deployAMMExchanges } from '@sienna/exchange'
import { AMMSNIP20Contract } from '@sienna/amm-snip20'
import { LPTokenContract } from '@sienna/lp-token'
import { SiennaSNIP20Contract } from '@sienna/snip20-sienna'
import { LaunchpadContract } from '@sienna/launchpad'
import { IDOContract } from '@sienna/ido'
import { AMMFactoryContract } from './FactoryContract'

/** Taking a TGE deployment, add the AMM to it,
  * creating the pre-configured liquidity and reward pools. */
export async function deployAMM ({
  deployment, agent, run,
  SIENNA  = deployment.getThe('SiennaSNIP20', new SiennaSNIP20Contract({agent})),
  version = 'v2',
}) {
  const {
    FACTORY
  } = await run(deployAMMFactory, { version })
  const {
    TOKENS, EXCHANGES, LP_TOKENS
  } = await run(deployAMMExchanges, { SIENNA, FACTORY, version })
  console.log()
  console.info(bold('Deployed AMM contracts:'))
  printContracts([FACTORY,...EXCHANGES,...LP_TOKENS])
  console.log()
  return { FACTORY, TOKENS, EXCHANGES, LP_TOKENS }
}

deployAMM.v1 = function deployAMM_v1 (args) {
  return deployAMM({ ...args, version: 'v1' })
}

deployAMM.v2 = function deployAMM_v2 (args) {
  return deployAMM({ ...args, version: 'v2' })
}

/** Deploy the Factory contract which is the hub of the AMM.
  * It needs to be passed code ids and code hashes for
  * the different kinds of contracts that it can instantiate.
  * So build and upload versions of those contracts too. */
export async function deployAMMFactory ({
  prefix, agent, chain, deployment,
  version = 'v2',
  suffix  = `@${version}+${timestamp()}`,
  copyFrom,
  initMsg = {
    admin:             agent.address,
    prng_seed:         randomHex(36),
    exchange_settings: getSettings(chain.id).amm.exchange_settings,
  }
}) {
  const options = { workspace, prefix, agent }
  const FACTORY   = new AMMFactoryContract({ ...options, version, suffix })
  const LAUNCHPAD = new LaunchpadContract({ ...options })
  // launchpad is new to v2 so we build/upload it every time...
  await chain.buildAndUpload(agent, [FACTORY, LAUNCHPAD])
  const template = contract => ({ id: contract.codeId, code_hash: contract.codeHash })
  if (copyFrom) {
    await deployment.createContract(agent, FACTORY, {
      ...initMsg,
      ...await copyFrom.getContracts(),
      // ...because otherwise here it wouldn've be able to copy it from v1...
      launchpad_contract: template(LAUNCHPAD),
    })
  } else {
    const [EXCHANGE, AMMTOKEN, LPTOKEN, IDO] = await chain.buildAndUpload(agent, [
      new AMMExchangeContract({ ...options, version }),
      new AMMSNIP20Contract({   ...options }),
      new LPTokenContract({     ...options }),
      new IDOContract({         ...options }),
    ])
    const contracts = {
      snip20_contract:    template(AMMTOKEN),
      pair_contract:      template(EXCHANGE),
      lp_token_contract:  template(LPTOKEN),
      ido_contract:       template(IDO),
      // ...while v1 here would just ignore this config field
      launchpad_contract: template(LAUNCHPAD),
    }
    await deployment.getOrCreateContract(agent, FACTORY, 'SiennaAMMFactory', {
      ...initMsg,
      ...contracts
    })
  }
  console.info(
    bold(`Deployed factory ${version}`), FACTORY.label
  )
  printContract(FACTORY)
  return { FACTORY }
}

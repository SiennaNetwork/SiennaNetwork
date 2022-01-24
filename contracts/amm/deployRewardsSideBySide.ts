import { Migration } from '@hackbg/fadroma'
import { RPTContract } from '@sienna/api'
import { deployRewards } from './deployRewards'

export async function deployRewardsSideBySide (migration: Migration) {

  const {

    timestamp,
    run,

    chain,
    admin,
    prefix,

  } = migration

  const options = { chain, admin, prefix }
  const v2Suffix = `@v2+${timestamp()}`
  const v3Suffix = `@v3+${timestamp()}`
  const v2 = await run(deployRewards('v2', { ...options, suffix: v2Suffix, split: 0.5, ref: 'rewards-2.1.2' })
  const v3 = await run(deployRewards('v3', { ...options, suffix: v3Suffix, split: 0.5, ref: 'HEAD' })
  const rptConfig = [
    ...v2.rptConfig,
    ...v3.rptConfig
  ]
  const RPT = chain.deployments.active.getContract(RPTContract, 'SiennaRPT', admin)
  await RPT.tx(admin).configure(rptConfig)
  console.log({rptConfig})
  console.table([
    ...v2.deployedContracts,
    ...v3.deployedContracts
  ].reduce((table, contract)=>{
    table[contract.init.label] = {
      address:  contract.init.address,
      codeId:   contract.blob.codeId,
      codeHash: contract.blob.codeHash
    }
    return table
  }, {}))
}

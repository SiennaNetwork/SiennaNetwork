import { IChain, bold, colors } from '@fadroma/scrt'

export function printRewardsContracts (chain: IChain) {

  if (chain && chain.deployments.active) {

    const {name, contracts} = chain.deployments.active
    const isRewardPool = (x: string) => x.startsWith('SiennaRewards_')
    const rewardsContracts = Object.keys(contracts).filter(isRewardPool)
    if (rewardsContracts.length > 0) {
      console.log(`\nRewards contracts in ${bold(name)}:`)
      for (const name of rewardsContracts) {
        console.log(`  ${colors.green('✓')}  ${name}`)
      }
    } else {
      console.log(`\nNo rewards contracts.`)
    }

  } else {

    console.log(`\nSelect a deployment to pick a reward contract.`)

  }

}

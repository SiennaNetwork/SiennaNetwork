import {
  InterestModelContract,
  LendMarketContract,
  LendOracleContract,
  LendOverseerContract
} from '@sienna/api'

import { workspace } from '@sienna/settings'

export async function buildLend (): Promise<string[]> {
  return Promise.all([
    new InterestModelContract({ workspace }).buildInDocker(),
    new LendMarketContract({    workspace }).buildInDocker(),
    new LendOracleContract({    workspace }).buildInDocker(),
    new LendOverseerContract({  workspace }).buildInDocker(),
  ])
}

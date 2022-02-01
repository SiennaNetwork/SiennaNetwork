import {
  InterestModelContract,
  LendMarketContract,
  LendOracleContract,
  LendOverseerContract,
  MockOracleContract
} from '@sienna/api'

import { workspace } from '@sienna/settings'

export async function buildLend (): Promise<string[]> {
  return Promise.all([
    new InterestModelContract({ workspace }).build(),
    new LendMarketContract({    workspace }).build(),
    new LendOracleContract({    workspace }).build(),
    new LendOverseerContract({  workspace }).build(),
    new MockOracleContract({workspace}).build(),
  ])
}

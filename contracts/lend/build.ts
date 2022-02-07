import { Artifact } from '@hackbg/fadroma'

import {
  InterestModelContract,
  LendMarketContract,
  LendOracleContract,
  LendOverseerContract,
  MockOracleContract
} from '@sienna/api'

import { workspace } from '@sienna/settings'

export async function buildLend (): Promise<Artifact[]> {
  return Promise.all([
    new InterestModelContract().build(),
    new LendMarketContract().build(),
    new LendOracleContract().build(),
    new LendOverseerContract().build(),
    new MockOracleContract().build(),
  ])
}

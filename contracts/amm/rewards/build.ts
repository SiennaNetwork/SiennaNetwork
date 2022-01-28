import { RewardsContract } from '@sienna/api'
import { workspace } from '@sienna/settings'
export async function buildRewards (): Promise<string[]> {
  return Promise.all([
    new RewardsContract({ workspace }).build(),
  ])
}

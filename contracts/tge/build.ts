import {
  SiennaSNIP20Contract,
  MGMTContract,
  RPTContract,
} from '@sienna/api'

import { workspace } from '@sienna/settings'

export async function buildTge (): Promise<string[]> {
  return Promise.all([
    new SiennaSNIP20Contract({ workspace }).build(),
    new MGMTContract({         workspace }).build(),
    new RPTContract({          workspace }).build()
  ])
}

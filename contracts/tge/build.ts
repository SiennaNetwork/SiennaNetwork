import {
  SiennaSNIP20Contract,
  MGMTContract,
  RPTContract,
} from '@sienna/api'

import { workspace } from '@sienna/settings'

export async function buildTge (): Promise<string[]> {
  return Promise.all([
    new SiennaSNIP20Contract({ workspace }).buildInDocker(),
    new MGMTContract({ workspace }).buildInDocker(),
    new RPTContract({ workspace }).buildInDocker()
  ])
}

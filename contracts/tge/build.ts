import {
  SiennaSNIP20Contract,
  MGMTContract,
  RPTContract,
} from '@sienna/api'

export async function buildTge (): Promise<string[]> {
  return Promise.all([
    new SiennaSNIP20Contract().build(),
    new MGMTContract().build(),
    new RPTContract().build()
  ])
}

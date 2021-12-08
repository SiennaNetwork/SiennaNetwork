import type { ContractUpload } from '@fadroma/ops'

export default async function buildAndUpload (contracts: Array<ContractUpload>) {
  await Promise.all(contracts.map(contract=>contract.build()))
  for (const contract of contracts) {
    await contract.upload()
  }
}


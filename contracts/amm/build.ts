import {
  FactoryContract,
  AMMContract,
  AMMSNIP20Contract,
  LPTokenContract,
  RewardsContract,
  SwapRouterContract,
  IDOContract,
  LaunchpadContract,
} from '@sienna/api'

export async function buildTokens (): Promise<string[]> {
  return Promise.all([
    new AMMSNIP20Contract().build(),
    new LPTokenContract().build(),
  ])
}

export async function buildAmm (): Promise<string[]> {
  return Promise.all([
    new FactoryContract().build(),
    new AMMContract().build(),
  ])
}

export async function buildIdo (): Promise<string[]> {
  return Promise.all([
    new IDOContract().build(),
    new LaunchpadContract().build(),
  ])
}

export async function buildRewards (): Promise<string[]> {
  return Promise.all([
    new RewardsContract().build(),
  ])
}

export async function buildRouter (): Promise<string[]> {
  return Promise.all([
    new SwapRouterContract().build()
  ])
}

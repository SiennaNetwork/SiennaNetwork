import { workspace } from '@sienna/settings'

import {
  AMMFactoryContract,
  AMMExchangeContract,
  AMMSNIP20Contract,
  LPTokenContract,
  RewardsContract,
  SwapRouterContract,
  IDOContract,
  LaunchpadContract,
} from '@sienna/api'

export * from './rewards/build'

export async function buildTokens (): Promise<string[]> {
  return Promise.all([
    new AMMSNIP20Contract({ workspace }).build(),
    new LPTokenContract({   workspace }).build(),
  ])
}

export async function buildAmm (): Promise<string[]> {
  return Promise.all([
    new AMMFactoryContract({  workspace }).build(),
    new AMMExchangeContract({ workspace }).build(),
    new AMMSNIP20Contract({   workspace }).build(),
  ])
}

export async function buildIdo (): Promise<string[]> {
  return Promise.all([
    new IDOContract({       workspace }).build(),
    new LaunchpadContract({ workspace }).build(),
  ])
}

export async function buildRouter (): Promise<string[]> {
  return Promise.all([
    new SwapRouterContract({ workspace }).build()
  ])
}

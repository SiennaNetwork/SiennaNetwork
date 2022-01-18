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

import { workspace } from '@sienna/settings'

export async function buildTokens (): Promise<string[]> {
  return Promise.all([
    new AMMSNIP20Contract({ workspace }).build(),
    new LPTokenContract({ workspace }).build(),
  ])
}

export async function buildAmm (): Promise<string[]> {
  return Promise.all([
    new FactoryContract({ workspace }).build(),
    new AMMContract({ workspace }).build(),
  ])
}

export async function buildIdo (): Promise<string[]> {
  return Promise.all([
    new IDOContract({ workspace }).build(),
    new LaunchpadContract({ workspace }).build(),
  ])
}

export async function buildRewards (): Promise<string[]> {
  return Promise.all([
    new RewardsContract({ workspace }).build(),
  ])
}

export async function buildRouter (): Promise<string[]> {
  return Promise.all([
    new SwapRouterContract({ workspace }).build()
  ])
}

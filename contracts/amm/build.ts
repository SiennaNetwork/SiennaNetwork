import type { Artifact } from '@hackbg/fadroma'

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

export async function buildTokens (): Promise<Artifact[]> {
  return Promise.all([
    new AMMSNIP20Contract().build(),
    new LPTokenContract().build(),
  ])
}

export async function buildAmm (): Promise<Artifact[]> {
  return Promise.all([
    new AMMFactoryContract['v1']().build(),
    new AMMFactoryContract['v2']().build(),
    new AMMExchangeContract['v1']().build(),
    new AMMExchangeContract['v2']().build(),
    new AMMSNIP20Contract().build(),
  ])
}

export async function buildIdo (): Promise<Artifact[]> {
  return Promise.all([
    new IDOContract().build(),
    new LaunchpadContract().build(),
  ])
}

export async function buildRouter (): Promise<Artifact[]> {
  return Promise.all([
    new SwapRouterContract().build()
  ])
}

export async function buildLatestAMMAndRewards (): Promise<Artifact[]> {
  return Promise.all([
    new AMMFactoryContract['v2']().build(),
    new AMMExchangeContract['v2']().build(),
    new LPTokenContract['v2']().build(),
    new RewardsContract['v3']().build(),
  ])
}

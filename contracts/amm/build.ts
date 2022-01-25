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
    new AMMSNIP20Contract({ workspace }).buildInDocker(),
    new LPTokenContract({ workspace }).buildInDocker(),
  ])
}

export async function buildAmm (): Promise<string[]> {
  return Promise.all([
    new FactoryContract({ workspace }).buildInDocker(),
    new AMMContract({ workspace }).buildInDocker(),
  ])
}

export async function buildIdo (): Promise<string[]> {
  return Promise.all([
    new IDOContract({ workspace }).buildInDocker(),
    new LaunchpadContract({ workspace }).buildInDocker(),
  ])
}

export async function buildRewards (): Promise<string[]> {
  return Promise.all([
    new RewardsContract({ workspace }).buildInDocker(),
  ])
}

export async function buildRouter (): Promise<string[]> {
  return Promise.all([
    new SwapRouterContract({ workspace }).buildInDocker()
  ])
}

import { Agent, ContractCaller as Base, ContractAPI } from '@hackbg/fadroma'
import { randomHex } from '@hackbg/fadroma'
import { SNIP20Contract, MGMTContract, RPTContract } from '@sienna/api'
import { abs } from './index'

// TGE /////////////////////////////////////////////////////////////////////////////////////////////

export class SiennaSNIP20 extends SNIP20Contract {
  code = { workspace: abs(), crate: 'snip20-sienna' }
  init = { label: 'SiennaSNIP20', msg: {
    get prng_seed () { return randomHex(36) },
    name:     "Sienna",
    symbol:   "SIENNA",
    decimals: 18,
    config:   { public_total_supply: true } } } }
export class MGMT extends MGMTContract {
  code = { workspace: abs(), crate: 'sienna-mgmt' }
  init = { label: 'SiennaMGMT', msg: {} } }
export class RPT extends RPTContract {
  code = { workspace: abs(), crate: 'sienna-rpt' }
  init = { label: 'SiennaRPT', msg: {} } }

// Swap ////////////////////////////////////////////////////////////////////////////////////////////

export class AMMFactory extends Base {
  code = { workspace: abs(), crate: 'factory' }
  init = { label: 'SiennaAMMFactory', msg: {} } }
export class AMMExchange extends Base {
  code = { workspace: abs(), crate: 'exchange' }
  init = { label: 'SiennaAMMExchange', msg: {} } }
export class AMMSNIP20 extends SNIP20Contract {
  code = { workspace: abs(), crate: 'amm-snip20' }
  init = { label: 'ExchangedSnip20', msg: {} } }

// Rewards /////////////////////////////////////////////////////////////////////////////////////////

const lpTokenDefaultConfig = {
  enable_deposit: true, enable_redeem: true,
  enable_mint: true, enable_burn: true,
  public_total_supply: true }
export class LPToken extends SNIP20Contract {
  code = { workspace: abs(), crate: 'lp-token' }
  init = { label: `LP`, msg: {
    get prng_seed () { return randomHex(36) },
    name:     "Liquidity Provision Token",
    symbol:   "LP",
    decimals: 18,
    config:   { ...lpTokenDefaultConfig } } }
  constructor (agent: Agent, name: string) {
    super(agent)
    this.init.label      = `SiennaRewards_${name}_LPToken`
    this.init.msg.symbol = `LP-${name}`
    this.init.msg.name   = `${name} liquidity provision token` }}

const BLOCK_TIME = 6 // seconds (on average)
const threshold  = 24 * 60 * 60 / BLOCK_TIME
const cooldown   = 24 * 60 * 60 / BLOCK_TIME
export class RewardPool extends Base {
  code = { workspace: abs(), crate: 'sienna-rewards' }
  init = { label: 'Rewards', msg: {
    threshold,
    cooldown,
    get viewing_key () { return randomHex(36) } } }
  constructor (agent: Agent, name: string) {
    super(agent)
    this.init.label = `SiennaRewards_${name}_Pool` }}

export function rewardPools (agent: Agent, pairs: Array<string>) {
  const pools = {}
  for (const pair of pairs) {
    pools[`LP_${pair}`] = new LPToken(agent, pair)
    pools[`RP_${pair}`] = new RewardPool(agent, pair) }
  return pools }

// IDO /////////////////////////////////////////////////////////////////////////////////////////////

export class IDO extends Base {
  code = { workspace: abs(), crate: 'ido' }
  init = { label: 'SiennaIDO', msg: {} } }

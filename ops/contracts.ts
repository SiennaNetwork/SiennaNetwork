import { Agent, ContractCaller as Base, ContractAPI } from '@fadroma/ops'
import { randomHex } from '@fadroma/tools'
import { SNIP20Contract, MGMTContract, RPTContract
       , FactoryContract, AMMContract
       , RewardsContract
       , IDOContract } from '@sienna/api'
import { abs } from './index'

// TGE /////////////////////////////////////////////////////////////////////////////////////////////

export class SiennaSNIP20 extends SNIP20Contract {
  code = { ...super.code, workspace: abs(), crate: 'snip20-sienna' }
  init = { ...super.init, label: 'SiennaSNIP20', msg: {
    get prng_seed () { return randomHex(36) },
    name:     "Sienna",
    symbol:   "SIENNA",
    decimals: 18,
    config:   { public_total_supply: true } } } }
export class MGMT extends MGMTContract {
  code = { ...super.code, workspace: abs(), crate: 'sienna-mgmt' }
  init = { ...super.init, label: 'SiennaMGMT', msg: {} } }
export class RPT extends RPTContract {
  code = { ...super.code, workspace: abs(), crate: 'sienna-rpt' }
  init = { ...super.init, label: 'SiennaRPT', msg: {} } }

// Swap ////////////////////////////////////////////////////////////////////////////////////////////

export class AMMFactory extends FactoryContract {
  code = { ...super.code, workspace: abs(), crate: 'factory' }
  init = { ...super.init, label: 'SiennaAMMFactory', msg: {
    get prng_seed () { return randomHex(36) },
    exchange_settings: { swap_fee:   { nom: 28, denom: 1000 }
                       , sienna_fee: { nom: 2, denom: 10000 }
                       , sienna_burner: null } } } }

export class AMMExchange extends AMMContract {
  code = { ...super.code, workspace: abs(), crate: 'exchange' }
  init = { ...super.init, label: 'SiennaAMMExchange', msg: {} } }

export class AMMSNIP20 extends SNIP20Contract {
  code = { ...super.code, workspace: abs(), crate: 'amm-snip20' }
  init = { ...super.init, label: 'ExchangedSnip20', msg: {} } }

// Rewards /////////////////////////////////////////////////////////////////////////////////////////

const lpTokenDefaultConfig = {
  enable_deposit: true, enable_redeem: true,
  enable_mint: true, enable_burn: true,
  public_total_supply: true }
export class LPToken extends SNIP20Contract {
  code = { ...super.code, workspace: abs(), crate: 'lp-token' }
  init = { ...super.init, label: `LP`, msg: {
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
export class RewardPool extends RewardsContract {
  code = { ...super.code, workspace: abs(), crate: 'sienna-rewards' }
  init = { ...super.init, label: 'Rewards', msg: {
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

export class IDO extends IDOContract {
  code = { ...super.code, workspace: abs(), crate: 'ido' }
  init = { ...super.init, label: 'SiennaIDO', msg: {} } }

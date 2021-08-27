import { BaseContractAPI as Base } from '@hackbg/fadroma'
import { randomHex } from '@hackbg/fadroma'
import { abs } from './index'

// TGE /////////////////////////////////////////////////////////////////////////////////////////////

export class SiennaSNIP20 extends Base {
  code = { workspace: abs(), crate: 'snip20-sienna' }
  init = { label: 'SiennaSNIP20', msg: {
    get prng_seed () { return randomHex(36) },
    name:     "Sienna",
    symbol:   "SIENNA",
    decimals: 18,
    config:   { public_total_supply: true } } } }
export class MGMT extends Base {
  code = { workspace: abs(), crate: 'sienna-mgmt' }
  init = { label: 'SiennaMGMT', msg: {} } }
export class RPT extends Base {
  code = { workspace: abs(), crate: 'sienna-rpt' }
  init = { label: 'SiennaRPT', msg: {} } }

// Swap ////////////////////////////////////////////////////////////////////////////////////////////

export class AMMFactory extends Base {
  code = { workspace: abs(), crate: 'factory' }
  init = { label: 'SiennaAMMFactory', msg: {} } }
export class AMMExchange extends Base {
  code = { workspace: abs(), crate: 'exchange' }
  init = { label: 'SiennaAMMExchange', msg: {} } }
export class AMMSNIP20 extends Base {
  code = { workspace: abs(), crate: 'amm-snip20' }
  init = { label: 'ExchangedSnip20', msg: {} } }

// Rewards /////////////////////////////////////////////////////////////////////////////////////////

const lpTokenDefaultConfig = {
  enable_deposit: true, enable_redeem: true,
  enable_mint: true, enable_burn: true,
  public_total_supply: true }
export abstract class LPToken extends Base {
  code = { workspace: abs(), crate: 'lp-token' } }
export abstract class RewardPool extends Base {
  code = { workspace: abs(), crate: 'sienna-rewards' } }

export class LP_SIENNA extends LPToken {
  init = { label: `LP_SIENNA`, msg: {
    get prng_seed () { return randomHex(36) },
    name: "SIENNA Liquidity Provision Token",
    symbol: "LP_SIENNA", decimals: 18,
    config: { ...lpTokenDefaultConfig } } } }
export class RewardPool_SIENNA extends RewardPool {
  init = { label: `RewardPool_SIENNA`, msg: {
    get entropy     () { return randomHex(36) },
    get prng_seed   () { return randomHex(36) },
    get viewing_key () { return randomHex(36) } } } }

export class LP_SIENNA_sSCRT extends LPToken {
  init = { label: `LP_SIENNA_sSCRT`, msg: {
    get prng_seed () { return randomHex(36) },
    name: "SIENNA/sSCRT Liquidity Provision Token",
    symbol: "LP_SIENNA_sSCRT", decimals: 18,
    config: { ...lpTokenDefaultConfig } } } }
export class RewardPool_SIENNA_sSCRT extends RewardPool {
  init = { label: `RewardPool_SIENNA_sSCRT`, msg: {
    get entropy     () { return randomHex(36) },
    get prng_seed   () { return randomHex(36) },
    get viewing_key () { return randomHex(36) } } } }

export class LP_SITOK_STEST extends LPToken {
  init = { label: `LP_SITOK_STEST`, msg: {
    get prng_seed () { return randomHex(36) },
    name: "SITOK/STEST Liquidity Provision Token",
    symbol: "LP_SITOK_STEST", decimals: 18,
    config: { ...lpTokenDefaultConfig } } } }
export class RewardPool_SITOK_STEST extends RewardPool {
  init = { label: `RewardPool_SITOK_STEST`, msg: {
    get entropy     () { return randomHex(36) },
    get prng_seed   () { return randomHex(36) },
    get viewing_key () { return randomHex(36) } } } }

export class LP_SIENNA_STEST extends LPToken {
  init = { label: `LP_SIENNA_STEST`, msg: {
    get prng_seed () { return randomHex(36) },
    name: "SIENNA/STEST Liquidity Provision Token",
    symbol: "LP_SIENNA_STEST", decimals: 18,
    config: { ...lpTokenDefaultConfig } } } }
export class RewardPool_SIENNA_STEST extends RewardPool {
  init = { label: `RewardPool_SIENNA_STEST`, msg: {
    get entropy     () { return randomHex(36) },
    get prng_seed   () { return randomHex(36) },
    get viewing_key () { return randomHex(36) } } } }

export class LP_SIENNA_SITOK extends LPToken {
  init = { label: `LP_SIENNA_SITOK`, msg: {
    get prng_seed () { return randomHex(36) },
    name: "SIENNA_SITOK Liquidity Provision Token",
    symbol: "LP_SIENNA_SITOK", decimals: 18,
    config: { ...lpTokenDefaultConfig } } } }
export class RewardPool_SIENNA_SITOK extends RewardPool {
  init = { label: `RewardPool_SIENNA_SITOK`, msg: {
    get entropy     () { return randomHex(36) },
    get prng_seed   () { return randomHex(36) },
    get viewing_key () { return randomHex(36) } } } }

export class LP_SIENNA_sETH extends LPToken {
  init = { label: `LP_SIENNA_sETH`, msg: {
    get prng_seed () { return randomHex(36) },
    name: "LP_SIENNA_sETH Liquidity Provision Token",
    symbol: "LP_SIENNA_sETH", decimals: 18,
    config: { ...lpTokenDefaultConfig } } } }
export class RewardPool_SIENNA_sETH extends RewardPool {
  init = { label: `RewardPool_SIENNA_sETH`, msg: {
    get entropy     () { return randomHex(36) },
    get prng_seed   () { return randomHex(36) },
    get viewing_key () { return randomHex(36) } } } }

export class LP_sSCRT_SITEST extends LPToken {
  init = { label: `LP_sSCRT_SITEST`, msg: {
    get prng_seed () { return randomHex(36) },
    name: "LP_sSCRT_SITEST Liquidity Provision Token",
    symbol: "LP_sSCRT_SITEST", decimals: 18,
    config: { ...lpTokenDefaultConfig } } } }
export class RewardPool_sSCRT_SITEST extends RewardPool {
  init = { label: `RewardPool_SIENNA_sSCRT`, msg: {
    get entropy     () { return randomHex(36) },
    get prng_seed   () { return randomHex(36) },
    get viewing_key () { return randomHex(36) } } } }

// IDO /////////////////////////////////////////////////////////////////////////////////////////////

export class IDO extends Base {
  code = { workspace: abs(), crate: 'ido' }
  init = { label: 'SiennaIDO', msg: {} } }

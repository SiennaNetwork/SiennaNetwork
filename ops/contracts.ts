import { randomHex } from '@hackbg/fadroma'

export const SIENNA_SNIP20 = {
  crate: 'snip20-sienna',
  label: `SiennaSNIP20`,
  initMsg: {
    get prng_seed () { return randomHex(36) },
    name:     "Sienna",
    symbol:   "SIENNA",
    decimals: 18,
    config:   { public_total_supply: true } } }

export const MGMT = {
  crate: 'sienna-mgmt',
  label: `SiennaMGMT`,
  initMsg: {} }

export const RPT = {
  crate: 'sienna-rpt',
  label: `SiennaRPT`,
  initMsg: {} }

export const AMM_FACTORY = {
  crate: 'factory',
  label: 'SiennaAMMFactory' }

export const AMM_EXCHANGE = {
  crate: 'exchange',
  label: 'SiennaAMMExchange' }

export const AMM_SNIP20 = {
  crate: 'amm-snip20',
  label: 'ExchangedSnip20' }

export const LP_SNIP20 = {
  crate: 'lp-token',
  label: `SiennaLPToken`,
  initMsg: {
    get prng_seed () { return randomHex(36) },
    name:     "Liquidity Provision Token",
    symbol:   "LPTKN",
    decimals: 18,
    config: {
      public_total_supply: true,
      enable_deposit:      true,
      enable_redeem:       true,
      enable_mint:         true,
      enable_burn:         true } } }

export const REWARD_POOL = {
  crate: 'sienna-rewards',
  label: `SiennaRewardPool`,
  initMsg: {
    get entropy     () { return randomHex(36) },
    get prng_seed   () { return randomHex(36) },
    get viewing_key () { return randomHex(36) } } }

export const IDO = {
  crate: 'ido',
  label: 'SiennaIDO' }

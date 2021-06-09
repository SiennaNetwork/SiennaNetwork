import { bignum } from '@fadroma/utilities'

export const fmtDecimals = d => x => {
  const a = bignum(x).div(d).toString()
  const b = bignum(x).mod(d).toString()
  return `${a}.${b.padEnd(18, '0')}`
}

export const SCRT_DECIMALS = 6
           , ONE_SCRT      = bignum(`1${[...Array(SCRT_DECIMALS)].map(()=>`0`).join('')}`)
           , fmtSCRT       = fmtDecimals(ONE_SCRT)

export const SIENNA_DECIMALS = 18
           , ONE_SIENNA      = bignum(`1${[...Array(SIENNA_DECIMALS)].map(()=>`0`).join('')}`)
           , fmtSIENNA       = fmtDecimals(ONE_SIENNA)

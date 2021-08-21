export const fmtDecimals = (d: number|string) => (x: number|string) => {
  const a = (BigInt(x) / BigInt(d)).toString()
  const b = (BigInt(x) % BigInt(d)).toString()
  return `${a}.${b.padEnd(18, '0')}` }

export const
  SCRT_DECIMALS = 6,
  ONE_SCRT = BigInt(`1${[...Array(SCRT_DECIMALS)].map(()=>`0`).join('')}`),
  fmtSCRT  = fmtDecimals(ONE_SCRT.toString())

export const
  SIENNA_DECIMALS = 18,
  ONE_SIENNA = BigInt(`1${[...Array(SIENNA_DECIMALS)].map(()=>`0`).join('')}`),
  fmtSIENNA  = fmtDecimals(ONE_SIENNA.toString())

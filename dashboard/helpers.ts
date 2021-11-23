// randomness helpers ------------------------------------------------------------------------------

export const random = (max: number) =>
  Math.floor(Math.random()*max)

export const pickRandom = (x: any) =>
  x[random(x.length)]

// timing helpers ----------------------------------------------------------------------------------

export function throttle (t: number, fn: Function) {
  // todo replacing t with a function allows for implementing exponential backoff
  let timeout: any
  return function throttled (...args:any) {
    return new Promise(resolve=>{
      if (timeout) clearTimeout(timeout)
      timeout = after(t, ()=>resolve(fn(...args))) })}}

export function after (t: number, fn: Function) {
  return setTimeout(fn, t) }

// DOM helpers -------------------------------------------------------------------------------------

export function h (element: string, attributes={}, ...content:any) {
  const el = Object.assign(document.createElement(element), attributes)
  for (const el2 of content) el.appendChild(el2)
  return el }

export const append = (parent: any) => (child: any) => parent.appendChild(child)

export function prepend (parent: any, child: HTMLElement) {
  return parent.insertBefore(child, parent.firstChild) }

// convert from string to Utf8Array ----------------------------------------------------------------

const enc = new TextEncoder()
export const encode = (x: any) => enc.encode(JSON.stringify(x))

const dec = new TextDecoder()
export const decode = (x: Uint8Array) => JSON.parse(dec.decode(x.buffer))

// -------------------------------------------------------------------------------------------------

export const DIGITS     = 1000
           , DIGITS_INV = Math.log10(DIGITS)
export const format = {
  integer:    (x:number) => String(x),
  decimal:    (x:number) => (x/DIGITS).toFixed(DIGITS_INV),
  percentage: (x:number) => `${format.decimal(x)}%`,
  SIENNA: (x:any) => `${BigInt(x)/1000000000000000000n} SIENNA`
}

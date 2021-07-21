import Gruvbox from './gruvbox'
import { Pool, User } from './contract_mock'

export const random = (max: number) =>
  Math.floor(Math.random()*max)

export const pickRandom = (x: any) =>
  x[random(x.length)]

export function throttle (t: number, fn: Function) {
  // todo replacing t with a function allows for implementing exponential backoff
  let timeout: any
  return function throttled (...args:any) {
    return new Promise(resolve=>{
      if (timeout) clearTimeout(timeout)
      timeout = after(t, ()=>resolve(fn(...args))) })}}

export function after (t: number, fn: Function) {
  return setTimeout(fn, t) }

export function h (element: string, attributes={}, ...content:any) {
  const el = Object.assign(document.createElement(element), attributes)
  for (const el2 of content) el.appendChild(el2)
  return el }

export function addTo (parent: HTMLElement, child: HTMLElement) {
  return parent.appendChild(child) }

export const COLORS = Object.assign(
  function getColor (pool: Pool, user: User) {
    switch (true) {
      case user.claimable > 0 && user.cooldown == 1: // have rewards to claim
        return COLORS.READY
      //case user.claimable > 0 && user.cooldown > 0: // just claimed, cooling down
        //return COLORS.ALL_OK
      case user.cooldown > 0:                       // waiting for age threshold
        return COLORS.COOLDOWN
      case user.claimable > pool.balance:           // not enough money in pool
        return COLORS.BLOCKED 
      case user.claimed > user.earned:              // crowded out
        return COLORS.CROWDED
      default:
        return COLORS.DEFAULT
    }
  }, {
    DEFAULT:  [Gruvbox.fadedAqua,   Gruvbox.light0],
    READY:    [Gruvbox.brightAqua,  Gruvbox.brightAqua],
    BLOCKED:  [Gruvbox.fadedOrange, Gruvbox.brightOrange],
    CROWDED:  [Gruvbox.fadedPurple, Gruvbox.brightPurple],
    COOLDOWN: [Gruvbox.fadedBlue,   Gruvbox.brightBlue],
    ALL_OK:   [Gruvbox.fadedAqua,   Gruvbox.brightAqua]
  })

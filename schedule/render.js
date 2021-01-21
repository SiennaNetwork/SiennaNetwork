#!/usr/bin/env node

const assert = require('assert')
const output = []

const Day = 24*60*60
const Month = 30*Day

const Invariants = {
  Node: 'node must be either recipient (addr) or category (streams), but not both',
  Vest: 'vesting must be be either monthly or daily, but not both',
  Time: 'time must be expressed as "<number>M" (where M stands for months)'
}

;(function descend (prefix, stream, {
  "%": percent,
  "=": amount,
  streams,
  addr,
  "cliff@": cliff_at,
  "cliff%": cliff_percent,
  monthly_over,
  daily_over
}) {
  console.log(`${prefix}${stream} ${percent}% = ${amount} SIENNA`)
  assert(!(streams&&addr), Invariants.Node)
  if (streams)
    for (let [stream, data] of Object.entries(streams))
      descend(prefix+' ', stream, data)
  else
    vest(`${prefix} ${addr} `, amount, cliff_at, cliff_percent, daily_over, monthly_over)
})('', 'total', require('./schedule'))

function vest (
  prefix,
  amount,
  cliff_at,
  cliff_percent,
  daily_over,
  monthly_over
) {

  cliff_at      = cliff_at      ? parseM(cliff_at)      : 0
  cliff_percent = cliff_percent ? (cliff_percent / 100) : 0

  assert(!(monthly_over&&daily_over), Invariants.Vest)

  if (daily_over) {
    daily_over = parseM(daily_over)
    for (let T = cliff_at; T <= cliff_at + daily_over; T += Day) {
      let tx
      if (cliff_percent > 0) {
        tx = amount * cliff_percent
        cliff_percent = 0
      } else {
        tx = amount * (1 / (daily_over / Day))
      }
      console.log(`${prefix}T+${T} ${tx}`)
      amount -= tx
    }

  } else if (monthly_over) {
    monthly_over = parseM(monthly_over)
    for (let T = cliff_at; T <= cliff_at + monthly_over; T += Day) {
      let tx
      if (cliff_percent > 0) {
        tx = amount * cliff_percent
        cliff_percent = 0
      } else {
        tx = amount * (1 / (monthly_over / Month))
      }
      console.log(`${prefix}T+${T} ${tx}`)
      amount -= tx
    }

  } else {
    throw new Error(Invariants.Vest)
  }

}

function parseM (x) {
  x = x.split('M')
  assert(x.length===2, Invariants.Time)
  x = Number(x[0])*Month
  assert(!isNaN(x), Invariants.Time)
  return x
}

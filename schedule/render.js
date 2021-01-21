#!/usr/bin/env node

const assert = require('assert')
const transactions = []

const Day = 24*60*60
const Month = 30*Day

const Invariants = {
  Node: 'node must be either recipient (addr) or category (streams), but not both',
  Vest: 'vesting must be be either monthly or daily, but not both',
  Time: 'time must be expressed as "<number>M" (where M stands for months)'
}

if (require.main === module) main()

function main () {
  descend('', 'total', require('./schedule'))
  console.log(transactions)
}

function descend (prefix, stream, {
  "%": percent,
  "=": amount,
  streams,
  addr,
  "cliff@": cliff_at,
  "cliff%": cliff_percent,
  monthly_over,
  daily_over
}) {
  console.log(`\n${prefix}${stream} ${percent}% = ${amount} SIENNA`)
  assert(!(streams&&addr), Invariants.Node)
  if (streams)
    for (let [stream, data] of Object.entries(streams))
      descend(prefix+'  ', stream, data)
  else
    vest(prefix+'  ', addr, amount, cliff_at, cliff_percent, daily_over, monthly_over)
}

function vest (
  prefix,
  addr,
  amount,
  cliff_at,
  cliff_percent,
  daily_over,
  monthly_over
) {

  cliff_at      = cliff_at      ? parseM(cliff_at)      : 0
  cliff_percent = cliff_percent ? (cliff_percent / 100) : 0
  if (cliff_at > 0 && cliff_percent > 0) {
    cliff = cliff_percent * amount
    console.log(
      `${prefix}T+${String(cliff_at).padEnd(10)} `+
      `${(String(cliff_percent*100)+'%').padStart(5)} `+
      `(${cliff.toFixed(14).padStart(22)} SIENNA) `+
      `-> ${addr}, then`)
    amount -= cliff

    transactions.push({T:cliff_at, sent:cliff, addr})
  }

  assert(!(monthly_over&&daily_over), Invariants.Vest)
  if (daily_over) {
    daily_over = parseM(daily_over)
    const days = Math.floor(daily_over / Day)
    console.log(`${prefix}daily over   ${String(days).padStart(3)} days:`)
    const daily = amount / days
    ;[...Array(days)].map((_,day)=>day+1).forEach(day=>{
      const T = cliff_at + day*Day
      const sent = Math.min(daily, amount)
      console.log(
        `${prefix}T+${String(T).padEnd(10)} `+
        `${`1/${days}`.padStart(5)} `+
        `(${sent.toFixed(14).padStart(22)} SIENNA) `+
        `-> ${addr}`)
      amount -= sent
      transactions.push({T, sent, addr})
    })
  } else if (monthly_over) {
    monthly_over = parseM(monthly_over)
    const months = Math.floor(monthly_over / Month)
    console.log(`${prefix}monthly over ${String(months).padStart(3)} months:`)
    const monthly = amount / months
    ;[...Array(months)].map((_,month)=>month+1).forEach((_, month)=>{
      const T = cliff_at + month*Month
      const sent = Math.min(monthly, amount)
      console.log(
        `${prefix}T+${String(T).padEnd(10)} `+
        `${`1/${months}`.padStart(5)} `+
        `(${sent.toFixed(14).padStart(22)} SIENNA) `+
        `-> ${addr}`)
      amount -= sent
      transactions.push({T, sent, addr})
    })
  } else {
    // release_immediately
    console.log(
      `${prefix}T+${String(0).padEnd(10)} `+
      `${`all`.padStart(5)} (${amount.toFixed(14).padStart(22)} SIENNA) `+
      `-> ${addr}`)
    amount -= amount
    transactions.push({T:0, sent:amount, addr})
  }

  console.log(`${prefix}Remaining: ${amount.toFixed(14)} SIENNA`)

}

function parseM (x) {
  x = x.split('M')
  assert(x.length===2, Invariants.Time)
  x = Number(x[0])*Month
  assert(!isNaN(x), Invariants.Time)
  return x
}

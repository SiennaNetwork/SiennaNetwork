#!/usr/bin/env node

const assert = require('assert')
const transactions = []

const Day = 24*60*60
const Month = 30*Day

const Invariants = {
  Node: 'node must be either recipient (addr) or category (streams), but not both',
  Vest: 'vesting must be be either monthly or daily, but not both',
  Time: 'time must be expressed as "<number>M" (where M stands for months)',
  Release: `release must be daily, montly, or immediate`
}

if (require.main === module) main()

function main () {
  require('fs').writeFileSync('transactions.txt', '')
  const data = require('./schedule')
  descend('', 'total', data)
  require('fs').writeFileSync('transactions.json', JSON.stringify(transactions, null, 2))
  render(transactions, data)
}

function log(...args) {
  console.log(...args)
  require('fs').appendFileSync('transactions.txt', '\n'+[...args].join(' '))
}

function descend (prefix, stream, {
  "%": percent,
  "=": amount,
  streams,
  addr,
  "cliff@": cliff_at,
  "cliff%": cliff_percent,
  release,
  over,
}) {
  log(`\n${prefix}${stream} ${percent}% = ${amount} SIENNA`)
  assert(!(streams&&addr), Invariants.Node)
  if (streams)
    for (let [stream, data] of Object.entries(streams))
      descend(prefix+'  ', stream, data)
  else
    vest(prefix+'  ', addr, amount, cliff_at, cliff_percent, release, over)
}

function vest (
  prefix,
  addr,
  amount,
  cliff_at,
  cliff_percent,
  release,
  over
) {

  cliff_at      = cliff_at      ? parseM(cliff_at)      : 0
  cliff_percent = cliff_percent ? (cliff_percent / 100) : 0

  if (cliff_at > 0 && cliff_percent > 0) {
    cliff = cliff_percent * amount
    log(
      `${prefix}T+${String(cliff_at).padEnd(10)} `+
      `${(String(cliff_percent*100)+'%').padStart(5)} `+
      `(${cliff.toFixed(14).padStart(22)} SIENNA) `+
      `-> ${addr}, then`)
    amount -= cliff

    transactions.push({T:cliff_at, sent:cliff, addr})
  }

  switch (release) {

    case 'immediate':
      const sent = amount
      log(
        `${prefix}T+${String(0).padEnd(10)} `+
        `${`all`.padStart(5)} `+
        `(${sent.toFixed(14).padStart(22)} SIENNA) `+
        `-> ${addr}`)
      amount -= sent
      transactions.push({T:0, sent, addr})
      break

    case 'daily':
      over = parseM(over)
      const days = Math.ceil(over / Day)
      log(`${prefix}daily over   ${String(days).padStart(3)} days:`)
      const daily = amount / days
      ;[...Array(days)].map((_,day)=>day+1).forEach(day=>{
        const T = cliff_at + day*Day
        const sent = Math.min(daily, amount)
        log(
          `${prefix}T+${String(T).padEnd(10)} `+
          `${`1/${days}`.padStart(5)} `+
          `(${sent.toFixed(14).padStart(22)} SIENNA) `+
          `-> ${addr}`)
        amount -= sent
        transactions.push({T, sent, addr})
      })
      break

    case 'monthly':
      over = parseM(over)
      const months = Math.ceil(over / Month)
      log(`${prefix}monthly over ${String(months).padStart(3)} months:`)
      const monthly = amount / months
      ;[...Array(months)].map((_,month)=>month+1).forEach(month=>{
        const T = cliff_at + month*Month
        const sent = Math.min(monthly, amount)
        log(
          `${prefix}T+${String(T).padEnd(10)} `+
          `${`1/${months}`.padStart(5)} `+
          `(${sent.toFixed(14).padStart(22)} SIENNA) `+
          `-> ${addr}`)
        amount -= sent
        transactions.push({T, sent, addr})
      })
      break

    default:
      throw new Error(Invariants.Release)
  }

  log(`${prefix}Remaining: ${amount.toFixed(14)} SIENNA`)

}

function parseM (x) {
  x = x.split('M')
  assert(x.length===2, Invariants.Time)
  x = Number(x[0])*Month
  assert(!isNaN(x), Invariants.Time)
  return x
}

function render (transactions, data) {
  const total = data["="]
  const balances_over_time = {
    'Contract': { 0: total }
  }
  const balances = {
    'Contract': total
  }
  const entries = []
  transactions = transactions.sort(({T:T1},{T:T2})=>T1-T2)
  let Tmin = 0, Tmax = 0
  for (let {T, sent, addr} of transactions) {
    if (T < Tmin) Tmin = T
    if (T > Tmax) Tmax = T
    //console.log({T, sent, addr})
    balances[addr] =
      balances[addr] || 0
    balances[addr] +=
      sent
    if (!balances_over_time[addr])
      balances_over_time[addr] = { 0: 0 }
    if (balances_over_time[addr][T])
      throw new Error(`2 transactions same addr same time`)
    balances_over_time[addr][T] =
      balances[addr]
    balances['Contract'] -=
      sent
    balances_over_time['Contract'][T] =
      balances['Contract']
  }

  console.log(balances_over_time)
  const accounts = []
  ;(function ascend (name, {addr, release, streams, "=":total}) {
    console.log('ascend', name, addr, streams)
    if (addr) {
      accounts.push({ addr, release, total, balances: balances_over_time[addr] })
    } else if (streams) {
      for (let [name, data] of Object.entries(streams)) {
        ascend(name, data)
      }
    }
  })('everything', data)

  require('fs').writeFileSync(
    'chart.svg', 
    require('pug').compileFile('chart.pug')({
      accounts,
      balances_over_time, Tmin, Tmax, total, data
    }))
}

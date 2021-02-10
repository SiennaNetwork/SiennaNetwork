#!/usr/bin/env node
const assert = require('assert')

const Sienna = x => {
  x = x.trim()
  if (x.length > 0) {
    return Number(x.replace(/,/g, ''))
  } 
}
const Percent = x =>
  x.replace(/%/g, '')
const Days = (x, y) => {
  x = Number(x.trim())
  assert(x*24*60*60 === Number(y), `${x} days must be accompanied with ${y} seconds`)
  return x
}
const Seconds = x =>
  Number(x.trim())
const Address = x => {
  x = x.trim()
  if (x.length > 0) {
    assert(x.length === 45, `address must be 45 characters: ${x}`)
    assert(x.startsWith('secret1'))
    return x
  }
}

const columns = ([
  _A_, _B_, _C_, _D_, _E_, _F_, _G_, _H_, _I_,
  _J_, _K_, _L_, _M_, _N_, _O_, _P_, _Q_
]) => {
  const data = {
    total:               Sienna  (_A_),
    pool:                String  (_B_),
    subtotal:            Sienna  (_C_),
    name:                String  (_D_),
    amount:              Sienna  (_E_),
    percent_of_total:    Percent (_F_),
    interval_days:       Days    (_G_, _H_),
    interval_seconds:    Seconds (_H_),
    start_at_days:       Days    (_I_, _J_),
    start_at:            Seconds (_J_),
    duration_days:       Days    (_K_, _L_),
    duration:            Seconds (_L_),
    cliff_percent:       Percent (_M_),
    cliff:               Sienna  (_N_),
    amount_per_interval: Sienna  (_O_),
    allocation:          Number  (_P_),
    address:             Address (_Q_)
  }
  return data
}

module.exports = function tsv2json (
  input = require('fs').readFileSync(`${__dirname}/../schedule.tsv`, 'utf8')
) {
  const output = {}
  let current_pool, current_release, current_allocation
  let running_total = 0
  let running_pool_total = 0
  let running_release_total = 0

  input
    .split('\n') // newline delimited
    .map(row=>row.split('\t')) // tab separated
    .map((data,i)=>[i+1,data]) // count rows from 1
    .forEach(([i, data]) => header(data, i)
                         || grand_total(columns(data), i)
                         || pool(columns(data), i)
                         || release(columns(data), i)
                         || allocation(columns(data), i)
                         || invalid_row(i))

  assert(running_total === output.total, `subtotals must add up to total`)

  return output

  function invalid_row (i) {
    console.warn(`row ${i}: skipping`)
  }

  function header (data, i) {
    if (i === 1) {
      // row is header, ignore it
      return true
    }
  }

  // if the row describes the grand total:
  function grand_total ({total, subtotal}, i) {
    if (i === 2) {
      assert(
        total===subtotal,
        'row 1 (schedule total): total must equal subtotal'
      )
      output.total = total
      output.pools = []
      console.log(`total: ${total}`)
      return true
    }
  }

  // if the row describes a pool:
  function pool (data, i) {
    let {pool, subtotal, name, percent_of_total} = data
    console.log('pool', data)
    if (pool && subtotal && percent_of_total) {
      assert(
        percent_of_total/100 === subtotal/output.total,
        `row ${i} (pool): percent_of_total=${percent_of_total} `+
        `must equal (subtotal[${subtotal} / total[${output.total}]) = ${subtotal/output.total}`
      )
      assert(
        (running_total = running_total + subtotal) <= output.total,
        `row ${i} (pool): subtotals must not add up to more than total`
      )
      if (current_pool) {
        assert(
          running_pool_total === current_pool.total,
          `row ${i} (pool): previous pool's subtotal was `+
          `${running_pool_total} (expected ${current_pool.total})`
        )
      }
      running_pool_total = 0
      output.pools.push(current_pool = {
        name: pool,
        total: subtotal,
        releases: []
      })
      console.log(`add pool ${pool} ${subtotal}`)
      return true
    }
  }

  // row describes release
  function release (data, i) {
    let {name,amount,percent_of_total,interval_days,interval
        ,amount_per_interval,address} = data
    if (name && amount && percent_of_total && interval) {
      amount = Number(amount.replace(/,/g, ''))
      if (interval) interval = Number(interval.trim())
      if (address) address = address.trim()
      const vesting = (interval == 0) ? { type: 'immediate' } : periodic_vesting(data, i)
      running_pool_total += amount
      running_release_total = Number(amount_per_interval)
      current_pool.releases.push(current_release = {
        name,
        amount,
        vesting,
        address,
        allocations: []
      })
      console.log(`add release ${name} (${address}) to pool ${current_pool.name}`)
      return true
    }
  }

  // 2nd part of release row
  // specifies parameters of periodic vesting
  function periodic_vesting (data, i) {
    const {interval,start_at,duration,cliff,amount_per_interval} = data
    const integer = (name, val) =>
      assert(!!val, `row ${i}: missing ${name}`)||Number(val)
    const percent = (name, val) =>
      assert(!!val, `row ${i}: missing ${name}`)||Number(val.replace(/%/g,'')/100)
    // TODO validate priors
    return {
      type:     'periodic',
      interval: integer('interval', interval),
      start_at: integer('start_at', start_at),
      duration: integer('duration', duration),
      cliff:    integer('cliff', cliff),
    }
  }

  function allocation (data, i) {
    let {allocation,address} = data
    if (allocation&&address) {
      // row describes allocation
      current_release.allocations.push({address,amount:allocation})
      running_release_total += allocation
    }
  }
}

if (require.main === module) require('fs').writeFileSync(
  `${__dirname}/../config_msg.json`,
  JSON.stringify(module.exports(), null, 2)
)

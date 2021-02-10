#!/usr/bin/env node
const assert = require('assert')

const ONE_SIENNA        = BigInt('1000000000000000000')
const THOUSANDTH_SIENNA = BigInt(   '1000000000000000')

const isN = x => !isNaN(x)

const Sienna = x => {
  x = x.trim()
  if (x.length > 0) {
    x = Number(x.replace(/,/g, ''))
    if (isN(x)) x = BigInt(x*1000) * THOUSANDTH_SIENNA
    return x
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
  _J_, _K_, _L_, _M_, _N_, _O_, _P_, _Q_, _R_
]) => {
  const data = {
    total:               Sienna  (_A_),
    pool:                String  (_B_),
    subtotal:            Sienna  (_C_),
    name:                String  (_D_),
    amount:              Sienna  (_E_),
    percent_of_total:    Percent (_F_),
    start_at_days:       Days    (_G_, _H_),
    start_at:            Seconds (_H_),
    interval_days:       Days    (_I_, _J_),
    interval:            Seconds (_J_),
    duration_days:       Days    (_K_, _L_),
    duration:            Seconds (_L_),
    vestings:            Number  (_M_),
    cliff_percent:       Percent (_N_),
    cliff:               Sienna  (_O_),
    amount_per_interval: Sienna  (_P_),
    allocation:          Sienna  (_Q_),
    address:             Address (_R_)
  }
  return data
}

module.exports = function tsv2json (
  input = require('fs').readFileSync(`${__dirname}/../schedule.tsv`, 'utf8')
) {
  const output = {}
  let current_pool
    , current_channel
    , current_allocation
    , running_total         = BigInt(0)
    , running_pool_total    = BigInt(0)
    , running_channel_total = BigInt(0)

  input
    .split('\n') // newline delimited
    .map(row=>row.split('\t')) // tab separated
    .map((data,i)=>[i+1,data]) // count rows from 1
    .forEach(([i, data]) => header(data, i)
                         || grand_total(columns(data), i)
                         || pool(columns(data), i)
                         || channel(columns(data), i)
                         || allocation(columns(data), i)
                         || invalid_row(columns(data), i))

  assert(running_total === output.total, `subtotals must add up to total`)

  return output

  function invalid_row (data, i) {
    console.warn(`row ${i}: skipping`, JSON.stringify(data))
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
      assert(total===subtotal,'row 1 (schedule total): total must equal subtotal')
      output.total = total
      output.pools = []
      console.log(`total: ${total}`)
      return true
    }
  }

  // if the row describes a pool:
  function pool (data, i) {
    let {pool, subtotal, name, percent_of_total} = data
    if (pool && subtotal && percent_of_total) {
      //assert(
        //percent_of_total/100 === subtotal/output.total,
        //`row ${i} (pool): percent_of_total=${percent_of_total} `+
        //`must equal (subtotal[${subtotal} / total[${output.total}]) = ${subtotal/output.total}`
      //)
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
      running_pool_total = BigInt(0)
      output.pools.push(current_pool = {
        name: pool,
        total: subtotal,
        partial: false,
        channels: []
      })
      console.log(`add pool ${pool} ${subtotal}`)
      return true
    }
  }

  // row describes channel
  function channel (data, i) {
    const {name,amount,percent_of_total,interval_days,interval
        ,amount_per_interval,address} = data
    if (name && amount && percent_of_total) {
      const mode = (interval == 0) ? { type: 'immediate' } : periodic_vesting(data, i)
      running_pool_total += amount
      running_channel_total = BigInt(amount_per_interval||0)
      current_pool.channels.push(current_channel = {
        name,
        amount,
        mode,
        allocations: []
      })
      if (address) current_channel.allocations.push({addr:address,amount:amount_per_interval||amount})
      console.log(`  add channel ${name} (${address}) to pool ${current_pool.name} (${running_pool_total})`)
      return true
    }
  }

  // 2nd part of channel row
  // specifies parameters of periodic vesting
  function periodic_vesting (data, i) {
    const {interval,start_at,duration,cliff,amount_per_interval} = data
    // TODO validate priors
    return {
      type: 'periodic',
      interval,
      start_at,
      duration,
      cliff
    }
  }

  function allocation (data, i) {
    let {allocation,address} = data
    if (allocation&&address) {
      // row describes allocation
      current_channel.allocations.push({addr:address,amount:allocation})
      running_channel_total += allocation
      return true
    }
  }
}

if (require.main === module) require('fs').writeFileSync(
  `${__dirname}/../config_msg.json`,
  require('json-bigint').stringify(module.exports(), (key, value) => (
      typeof value === 'bigint'
          ? value.toString()
          : value // return everything else unchanged
  ), 2)
)

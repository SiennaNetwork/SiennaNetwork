#!/usr/bin/env node
const assert = require('assert')
const row2record = require('./columns')

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
                         || grand_total(row2record(data), i)
                         || pool(row2record(data), i)
                         || channel(row2record(data), i)
                         || allocation(row2record(data), i)
                         || invalid_row(row2record(data), i))

  assert(running_total === output.total, `subtotals must add up to total`)

  return output

  function invalid_row (data, i) {
    console.warn(`row ${i}: skipping`, JSON.stringify(data, (key, value) => (
        typeof value === 'bigint'
            ? value.toString()
            : value // return everything else unchanged
    ), 2))
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
    const {name,amount,percent_of_total,interval_days,interval,expected_portion,address} = data
    if (name && amount && percent_of_total) {
      running_pool_total += amount
      running_channel_total = BigInt(expected_portion||0)
      current_pool.channels.push(current_channel = {
        name,
        amount,
        periodic: (interval == 0) ? undefined : periodic_vesting(data, i),
        allocations: [[0, []]]
      })
      if (address) current_channel.allocations[0][1].push({addr:address,amount:expected_portion||amount})
      console.log(`  add channel ${name} (${address}) to pool ${current_pool.name} (${running_pool_total})`)
      return true
    }
  }

  // 2nd part of channel row
  // specifies parameters of periodic vesting
  function periodic_vesting (data, i) {
    const {interval,start_at,duration,cliff,portion,expected_portion,expected_remainder} = data
    // TODO validate priors
    return {
      type: 'channel_periodic',
      interval, start_at, duration, cliff,
      expected_portion, expected_remainder
    }
  }

  function allocation (data, i) {
    let {allocation,address} = data
    if (allocation&&address) {
      // row describes allocation
      current_channel.allocations[0][1].push({addr:address,amount:allocation})
      running_channel_total += allocation
      return true
    }
  }
}

#!/usr/bin/env node
const assert = require('assert')

module.exports = function tsv2json (
  input = require('fs').readFileSync(`${__dirname}/../schedule.tsv`, 'utf8')
) {
  const data = {}
  let current_pool, current_account, current_allocation
  let running_total = 0
  let running_pool_total = 0
  let running_account_total = 0

  input
    .split('\n') // newline delimited
    .map(row=>row.split('\t') // tab separated
    .map((data,i)=>[i+1,data]) // count rows from 1
    .forEach(([i, data])
      => header(data, i)
      || grand_total(data, i)
      || pool(data, i)
      || account(data, i)
      || allocation(data, i)
      || throw new Error(`row ${i}: invalid row`))

  assert(
    running_total === data.total,
    `subtotals didn't add up to total`
  )

  return data

  function header (data, i) {
    if (i === 1) {
      // row is header, ignore it
      return true
    }
  }

  function grand_total ([total, _, subtotal], i) {
    if (i === 2) {
      // row describes the grand total total
      total = total.replace(/,/g, '')
      subtotal = subtotal.replace(/,/g, '')
      assert(
        total===subtotal,
        'row 1 (schedule total): total must equal subtotal'
      )
      data.total = total
      data.pools = []
      return true
    }
  }

  function pool ([
    _,
    pool,subtotal,
    __,___,
    percent_of_total
  ], i) {
    if (pool && subtotal && percent_of_total) {
      // row describes a pool
      assert(
        Number(percent_of_total.replace(/%/,''))/100 === subtotal/data.total,
        `row ${i} (pool): percent_of_total must equal (subtotal / total)`
      )
      assert(
        (running_total = running_total + subtotal) <= data.total,
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
      data.push(current_pool = {
        name: category,
        total: subtotal,
        accounts: []
      })
      return true
    }
  }

  function account (data, i) {
    const [
      _,__,___,name,amount,percent_of_total,interval,
      ____,_____,______,_______,________,address
    ] = data

    if (
      name && amount && percent_of_total && interval
    ) {
      // row describes account
      assert(!!address, `row ${i}: missing address`)
      let vesting =
        (interval === 'IMMEDIATE')
          ? { type: 'immediate' } :
        (interval === 'DAILY' && interval === 'MONTHLY')
          ? periodic_vesting(data, i) :
        throw new Error(`row ${i}: invalid interval: ${interval}`)
      if (current_account) {
        assert(
          running_account_total === current_account.amount,
          `row ${i} (pool): previous account's allocations added up to `+
          `${running_account_total} (expected ${current_account.amount})`
        )
      }
      running_account_total = amount_per_interval
      current_pool.accounts.push(current_account = {
        name: name
        amount: amount,
        vesting,
        recipients: [{address, amount_per_interval}]
      })
    }
  }

  function periodic_vesting (data, i) {
    const [
      _,__,___,____,_____,______,
      interval,start_after_x_days,duration_days,
      cliff_percent,cliff_amount,amount_per_interval,
    ] = data
    return {
      type: 'periodic',
      interval,
      start_after_x_days:
        assert(
          !!start_after_x_days,
          `row ${i}: missing start_after_x_days`
        ) || start_after_x_days,
      duration_days:
        assert(
          !!duration_days,
          `row ${i}: missing duration_days`
        ) || duration_days,
      cliff_percent:
        assert(
          !!cliff_percent,
          `row ${i}: missing cliff_percent`
        ) || cliff_percent,
      cliff_amount:
        assert(
          !!cliff_amount,
          `row ${i}: missing cliff_amount`
        ) || cliff_amount,
      amount_per_interval:
        assert(
          !!amount_per_interval,
          `row ${i}: missing amount_per_interval`
        ) || amount_per_interval,
    }
  }

  function allocation (data, i) {
    const [
      _,__,___,____,____,______,
      _______,________,_________,
      __________,___________,amount_per_interval,address
    ] = data
    if (amount_per_interval&&address) {
      // row describes allocation
      current_account.recipients.push({address, amount_per_interval})
      running_account_total += amount_per_interval
    }
  }
}

if (require.main === module) process.stdout.write(
  JSON.stringify(module.exports(), null, 2)
)

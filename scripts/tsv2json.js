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
    .map(row=>row.split('\t')) // tab separated
    .map((data,i)=>[i+1,data]) // count rows from 1
    .forEach(([i, data]) => header(data, i)
                         || grand_total(data, i)
                         || pool(data, i)
                         || account(data, i)
                         || allocation(data, i)
                         || invalid_row(i))

  assert(running_total === data.total, `subtotals must add up to total`)

  return data

  function invalid_row (i) {
    console.warn(`row ${i}: empty or invalid row, skipping`)
  }

  function header (data, i) {
    if (i === 1) {
      // row is header, ignore it
      return true
    }
  }

  function grand_total ([total, _B_, subtotal], i) {
    if (i === 2) {
      // row describes the grand total total
      total = Number(total.replace(/,/g, ''))
      subtotal = Number(subtotal.replace(/,/g, ''))
      assert(
        total===subtotal,
        'row 1 (schedule total): total must equal subtotal'
      )
      data.total = total
      data.pools = []
      console.log(`total: ${total}`)
      return true
    }
  }

  function pool ([
    _A_,pool,subtotal,_D_,_E_,percent_of_total
  ], i) {
    if (pool && subtotal && percent_of_total) {
      // row describes a pool
      subtotal = Number(subtotal.replace(/,/g, ''))
      assert(
        Number(percent_of_total.replace(/%/,''))/100 === subtotal/data.total,
        `row ${i} (pool): percent_of_total=${percent_of_total} `+
        `must equal (subtotal[${subtotal} / total[${data.total}]) = ${subtotal/data.total}`
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
      data.pools.push(current_pool = {
        name: pool,
        total: subtotal,
        accounts: []
      })
      console.log(`add pool ${pool} ${subtotal}`)
      return true
    }
  }

  function account (data, i) {
    let [
      _A_,_B_,_C_,name,amount,percent_of_total,interval,
      _H_,_I_,_J_,_K_,amount_per_interval,_M_,address
    ] = data

    if (name && amount && percent_of_total && interval) {
      // row describes account
      amount = Number(amount.replace(/,/g, ''))
      if (interval) interval = interval.trim()
      if (address) address = address.trim()
      let vesting
      if (interval === 'DAILY' || interval === 'MONTHLY') {
        vesting = periodic_vesting(data, i)
      } else if (interval === 'IMMEDIATE') {
        vesting = { type: 'immediate' }
      } else {
        throw new Error(`row ${i}: invalid vesting config`)
      }
      if (interval === 'IMMEDIATE')
        (interval === 'IMMEDIATE') ? { type: 'immediate' } :
        (interval === 'DAILY')     ? periodic_vesting(data, i) :
        (interval === 'MONTHLY')   ? periodic_vesting(data, i) :
        invalid_vesting(i)
      if (current_account && !current_account.address) {
        assert(
          running_account_total === current_account.amount,
          `row ${i} (pool): previous account's allocations added up to `+
          `${running_account_total} (expected ${current_account.amount_per_interval})`
        )
      }
      running_pool_total += amount
      running_account_total = amount_per_interval
      current_pool.accounts.push(current_account = {
        name,
        amount,
        vesting,
        address,
        recipients: []
      })
      console.log(`add account ${name} (${address}) to pool ${current_pool.name}`)
      return true
    }
  }

  function periodic_vesting (data, i) {
    // 2nd part of account row
    // specifies parameters of periodic vesting
    const [
      _A_,_B_,_C_,_D_,_E_,_F_,
      interval,start_after_x_days,duration_days,
      cliff_percent,cliff_amount,amount_per_interval,
    ] = data
    const integer = (name, val) =>
      assert(!!val, `row ${i}: missing ${name}`)||Number(val)
    const percent = (name, val) =>
      assert(!!val, `row ${i}: missing ${name}`)||Number(val.replace(/%/g,'')/100)
    return {
      type: 'periodic',
      interval,
      start_after_x_days:  integer('start_after_x_days', start_after_x_days),
      duration_days:       integer('duration_days', duration_days),
      cliff_percent:       percent('cliff_percent', cliff_percent),
      cliff_amount:        integer('cliff_amount', cliff_amount),
      amount_per_interval: amount_per_interval ? Number(amount_per_interval) : undefined
    }
  }

  function allocation (data, i) {
    let [
      _A_,_B_,_C_,_D_,_E_,_F_,_G_,_H_,_I_,_J_,_K_,_L_,
      allocation,address
    ] = data
    if (allocation&&address) {
      // row describes allocation
      address = address.trim()
      allocation = Number(allocation)
      current_account.recipients.push({address,amount:allocation})
      running_account_total += allocation
    }
  }
}

if (require.main === module) require('fs').writeFileSync(
  `${__dirname}/../config_msg.json`,
  JSON.stringify({configure:{schedule:module.exports()}}, null, 2)
)

const JSON = require('json-bigint')
const assert = require('assert')
const XLSX = require('xlsx')

module.exports = function scheduleFromSpreadsheet ({
  file,                                   // pass a filename
  book  = XLSX.readFile(file),            // or pass a Sheet.js Workbook object
  sheet = book.Sheets[book.SheetNames[0]] // or a Sheet.js Sheet
}) {

  // table boundaries
  const { s: { c: startCol, r: startRow }
        , e: { c: endCol,   r: endRow   }
        } = XLSX.utils.decode_range(sheet['!ref']) 

  // `+2` needed to include 1st and last columns?
  const columns = [...Array(2 + endCol - startCol)] 
    .map((_,i)=>XLSX.utils.encode_col(i))

  // data will be collected here
  const schedule = {} 

  // ignore these rows
  const headerHeight = 5
  let currentPool      = null
    , currentAccount   = null
    , runningTotal     = BigInt(0)
    , runningPoolTotal = BigInt(0)

  for (let i = startRow + headerHeight; // skip header
       i <= endRow + 1;                 // `+1` needed to count the last row?
       i++                              // go over every line
  ) {

    const row   = XLSX.utils.encode_row(i); // get row number
    const cells = columns.map(col=>`${col}${i}`).map(cell=>(sheet[cell]||{}).v) // get row values
    const data  = require('./columns')(cells) // turn [cell] to {field:value}

    // Grand total (first line after header)
    if (i === headerHeight) {
      console.log(
        `row ${String(i).padStart(3)}: ` +
        `total (${data.total})`
      )
      assert(
        data.total===data.subtotal,
        `row ${headerHeight} (schedule total): total must equal subtotal`
      )
      schedule.total = data.total
      schedule.pools = []
    }

    // Pool:
    else if (data.pool && data.subtotal && data.percent_of_total) {
      const {pool, subtotal} = data
      console.log(
        `\nrow ${String(i).padStart(3)}: `+
        `pool    ${String(pool).padEnd(15)}` +
        ` = ${String(subtotal).padStart(30)} ` +
        `(${Number(data.percent_of_total*10000).toFixed(2).padStart(8)}% of total)`)
      assert((runningTotal = runningTotal + subtotal) <= schedule.total,
        `row ${i} (pool): subtotals must not add up to more than total`)
      if (currentPool) {
        assert(runningPoolTotal <= currentPool.total,
          `row ${i} (pool): pool's subtotal was `+
          `${runningPoolTotal} (expected no more than ${currentPool.total})`)
        if (runningPoolTotal < currentPool.total) currentPool.partial = true
      }
      runningPoolTotal = BigInt(0) // reset running total when encountering each new pool
      schedule.pools.push(currentPool={name: pool, total: subtotal, partial: false, accounts: []})
      //console.debug(`add pool ${pool} ${subtotal}`)
    }

    // Account in above Pool
    else if (data.name && data.amount && data.percent_of_total) {
      runningPoolTotal += data.amount
      const {name,amount,percent_of_total,interval,portion_size,address} = data
      console.log(
        `row ${String(i).padStart(3)}: ` +
        `account ${String(name).padEnd(15)}`
      )
      currentPool.accounts.push(currentAccount = pick(data, [
        'name', 'amount', 'address',
        'start_at', 'interval', 'duration',
        'cliff', 'portion_size', 'remainder',
      ]))
    }

    // Other things
    else {
      //console.warn(`unknown line ${i}:`, data)
    }
  }
  return schedule
}

// BigInt-aware JSON serializer (converts them to strings)
function stringify (data) {
  return JSON.stringify(data, (key, value) => {
    return typeof value === 'bigint' ? value.toString() : value // return everything else unchanged
  }, 2)
}

function pick (obj = {}, keys = []) {
  return keys.reduce((result, key)=>Object.assign(result, {[key]:obj[key]}),{})
}

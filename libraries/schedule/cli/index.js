const JSON = require('json-bigint')
const assert = require('assert')
const XLSX = require('xlsx')

module.exports = {
  scheduleFromSpreadsheet,
  portionsFromSchedule,
  chartFromScheduleAndPortions,
  spreadsheetFromSchedule
}

function scheduleFromSpreadsheet ({
  file,                                   // pass a filename
  book  = XLSX.readFile(file),            // or pass a Sheet.js Workbook object
  sheet = book.Sheets[book.SheetNames[0]] // or a Sheet.js Sheet
}) {
  const { s: { c: startCol, r: startRow }
        , e: { c: endCol,   r: endRow   }
        } = XLSX.utils.decode_range(sheet['!ref']) // table boundaries
  const columns = [...Array(2 + endCol - startCol)] // `+2` needed to include 1st and last columns?
    .map((_,i)=>XLSX.utils.encode_col(i))
  //console.log({startCol, endCol}, columns)
  const schedule = {} // data will be added here
  let currentPool      = null
    , currentAccount   = null
    , runningTotal     = BigInt(0)
    , runningPoolTotal = BigInt(0)
  for (let i = startRow + 4; // skip header
       i <= endRow + 1;      // `+1` needed to count the last row?
       i++                   // go over every line
  ) {
    const row   = XLSX.utils.encode_row(i);
    const cells = columns.map(col=>`${col}${i}`).map(cell=>(sheet[cell]||{}).v)
    const data  = require('./columns')(cells)

    // Grand total (first line after header)
    if (i === 4) { 
      assert(data.total===data.subtotal,'row 4 (schedule total): total must equal subtotal')
      schedule.total = data.total
      schedule.pools = []
    }
    // Pool:
    else if (data.pool && data.subtotal && data.percent_of_total) {
      const {pool, subtotal} = data
      assert((runningTotal = runningTotal + subtotal) <= schedule.total,
        `row ${i} (pool): subtotals must not add up to more than total`)
      if (currentPool) assert(runningPoolTotal === currentPool.total,
        `row ${i} (pool): previous pool's subtotal was `+
        `${runningPoolTotal} (expected ${currentPool.total})`)
      runningPoolTotal = BigInt(0) // reset running total when encountering each new pool
      schedule.pools.push(currentPool={name: pool, total: subtotal, partial: false, accounts: []})
      //console.debug(`add pool ${pool} ${subtotal}`)
    }
    // Account in above Pool
    else if (data.name && data.amount && data.percent_of_total) {
      runningPoolTotal += data.amount
      const {name,amount,percent_of_total,interval,portion_size,address} = data
      currentPool.accounts.push(currentAccount = pick(data, [
        'name', 'amount',
        'start_at', 'interval', 'duration',
        'cliff', 'portion_size', 'remainder',
        'head_allocations',
        'body_allocations',
        'tail_allocations',
      ]))
    }
    // Other things
    else {
      //console.warn(`unknown line ${i}:`, data)
    }
  }
  return schedule
}

function portionsFromSchedule (schedule) {
  console.log('portionsFromSchedule: not implemented')
  process.exit(1)
}

function chartFromScheduleAndPortions ({ schedule, portions }) {
  console.log('chartFromScheduleAndPortions: not implemented')
  process.exit(1)
}

function spreadsheetFromSchedule () {
  console.log('spreadsheetFromSchedule: not implemented')
  process.exit(1)
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

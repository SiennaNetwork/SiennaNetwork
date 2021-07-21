import './dashboard/style.css'
import { Log, Table, PieChart, StackedPieChart } from './dashboard/widgets'
import { T, Pool, User } from './dashboard/contract_mock'
import { random, pickRandom, throttle, after, addTo } from './dashboard/helpers'
import initMock from './dashboard/contract_mock'

// settings ----------------------------------------------------------------------------------------
const UPDATE_INTERVAL = 0

// dashboard components ----------------------------------------------------------------------------
const log     = new Log()
const table   = new Table()
const current = new PieChart('Current amounts locked',  'locked')
const stacked = new StackedPieChart()
const ui = { log, table, current, stacked }

// the rewards contract and its participants -------------------------------------------------------
const {pool, users} = initMock(ui)
table.init(users)

// start on click ----------------------------------------------------------------------------------

document.body.onclick = () => {
  start()
  document.body.onclick = null
}

function start () {
  // add components
  for (const el of [ current
                   , stacked
                   //, lifetime
                   //, earned
                   //, claimed
                   , log
                   , table /*,sparkline*/]) {
    addTo(document.body, el.root) }

  resize()
  window.addEventListener('resize', throttle(100, resize))

  update()
  function update () {
    // advance time
    T.T++

    // periodically fund pool and increment its lifetime
    pool.update()

    // increment lifetimes and ages; collect eligible claimants
    const eligible: Array<User> = []
    for (const user of Object.values(users)) {
      user.update()
      if (user.claimable > 0) eligible.push(user)
    }

    // perform random lock/retrieve from random account for random amount
    const user   = pickRandom(Object.values(users))//Object.keys(users)[Math.floor(Math.random()*Object.keys(users).length)]
    const action = pickRandom([(amount:number)=>user.lock(amount)
                              ,(amount:number)=>user.retrieve(amount)])//Object.values(actions)[random(actions.length)]
    action(random(user.balance))

    // perform random claim
    if (eligible.length > 0) {
      const claimant = pickRandom(eligible)
      claimant.claim()
    }

    // update charts
    for (const chart of [current
                        ,stacked
                        //,lifetime
                        //,earned
                        //,claimed
                        ]) { chart.render() }

    // rinse and repeat
    after(UPDATE_INTERVAL, update) }
  function resize () {
    current.resize()
    stacked.resize()
    //lifetime.resize()
    //earned.resize()
    //claimed.resize()
  } }

import './dashboard/style.css'
import { Log, Table, PieChart, StackedPieChart } from './dashboard/widgets'
import { T, User } from './dashboard/contract_mock'
import { random, pickRandom, throttle, after, addTo } from './dashboard/helpers'
import initMock from './dashboard/contract_mock'
import initReal from './dashboard/contract_real'

// settings ----------------------------------------------------------------------------------------
const UPDATE_INTERVAL = 0

// dashboard components ----------------------------------------------------------------------------
const ui = {
  log:     new Log(),
  table:   new Table(),
  current: new PieChart('Current amounts locked',  'locked'),
  stacked: new StackedPieChart()
}

// the rewards contract and its participants -------------------------------------------------------
const {pool, users} = initMock(ui)
initReal(ui)
ui.table.init(users)

// start on click ----------------------------------------------------------------------------------
document.body.onclick = () => {
  start()
  document.body.onclick = null
}

function start () {
  // add components
  for (const el of Object.values(ui)) {
    addTo(document.body, el.root)
  }

  // add resize handler
  resize()
  window.addEventListener('resize', throttle(100, resize))

  // start updating
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
    for (const chart of [ui.current,ui.stacked]) {
      chart.render()
    }

    // rinse and repeat
    after(UPDATE_INTERVAL, update)
  }

  function resize () {
    ui.current.resize()
    ui.stacked.resize()
  }
}

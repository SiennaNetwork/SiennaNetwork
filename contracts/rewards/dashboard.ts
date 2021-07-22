import './dashboard/style.css'
import { Log, Table, PieChart, StackedPieChart } from './dashboard/widgets'
import { T, Users, MAX_USERS, MAX_INITIAL } from './dashboard/contract_base'
import { RealPool as Pool, RealUser as User } from './dashboard/contract_real'
import { random, pickRandom, throttle, after, addTo } from './dashboard/helpers'
//import initMock from './dashboard/contract_mock'
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
//const {pool, users} = initMock(ui)
let pool: Pool
let users: Users = {}
initReal().then(()=>{
  pool = new Pool(ui)

  // create a number of test users with random balances --------------------------------------------
  for (let i = 0; i < MAX_USERS; i++) {
    const name    = `User${i}`
    const balance = Math.floor(Math.random()*MAX_INITIAL)
    users[name]   = new User(ui, pool, name, balance)
  }

  // create dom elements for all users - then only update the content ------------------------------
  ui.table.init(users)

  // start on click --------------------------------------------------------------------------------
  document.body.onclick = () => {
    start()
    document.body.onclick = null
  }
})

function start () {
  // add components --------------------------------------------------------------------------------
  for (const el of Object.values(ui)) {
    addTo(document.body, el.root)
  }

  // add resize handler ----------------------------------------------------------------------------
  resize()
  window.addEventListener('resize', throttle(100, resize))

  // start updating --------------------------------------------------------------------------------
  update()
  function update () {
    // advance time --------------------------------------------------------------------------------
    T.T++

    // periodically fund pool and increment its lifetime -------------------------------------------
    pool.update()

    // increment lifetimes and ages; collect eligible claimants ------------------------------------
    const eligible: Array<User> = []
    for (const user of Object.values(users)) {
      user.update()
      if (user.claimable > 0) eligible.push(user as User)
    }

    // perform random lock/retrieve from random account for random amount --------------------------
    const user   = pickRandom(Object.values(users))
    const action = pickRandom([
      (amount:number)=>user.lock(amount),
      (amount:number)=>user.retrieve(amount)
    ])

    action(random(user.balance))

    // perform random claim ------------------------------------------------------------------------
    if (eligible.length > 0) {
      const claimant = pickRandom(eligible)
      claimant.claim()
    }

    // update charts -------------------------------------------------------------------------------
    for (const chart of [ui.current,ui.stacked]) {
      chart.render()
    }

    // rinse and repeat ----------------------------------------------------------------------------
    after(UPDATE_INTERVAL, update)
  }

  // resize handler --------------------------------------------------------------------------------
  function resize () {
    ui.current.resize()
    ui.stacked.resize()
  }
}

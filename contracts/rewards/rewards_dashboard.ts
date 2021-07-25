import './dashboard/style.css'
import { Log, Table, PieChart, StackedPieChart } from './dashboard/widgets'
import { T, Users, MAX_USERS, MAX_INITIAL } from './dashboard/contract_base'
import { RealPool as Pool, RealUser as User } from './dashboard/contract_real'
import { random, pickRandom, throttle, after, addTo } from './dashboard/helpers'
//import initMock from './dashboard/contract_mock'
import initReal from './dashboard/contract_real'

document.body.innerHTML = '<center>loading</center>'

// settings ----------------------------------------------------------------------------------------
const UPDATE_INTERVAL  = 1
const AUTO_CLAIM       = false
const AUTO_LOCK_UNLOCK = false

initReal().then(()=>{ // load then start on click --------------------------------------------------
  document.body.onclick = () => {
    document.body.innerHTML = ''
    document.body.onclick = null
    start()
  }
  document.body.innerHTML = '<center>click to start</center>'
})

function start () {

  // create the dashboard --------------------------------------------------------------------------
  const ui = {
    log:     new Log(),
    table:   new Table(),
    current: new PieChart('Current amounts locked',  'locked'),
    stacked: new StackedPieChart()
  }

  // create a pool and some of test users with random balances -------------------------------------
  const pool = new Pool(ui)
  const users: Users = {}
  for (let i = 0; i < MAX_USERS; i++) {
    const name    = `User${i}`
    const balance = Math.floor(Math.random()*MAX_INITIAL)
    users[name]   = new User(ui, pool, name, balance)
  }

  // add components --------------------------------------------------------------------------------
  for (const el of Object.values(ui)) {
    addTo(document.body, el.root)
  }

  // create dom elements for all users - then only update the content ------------------------------
  ui.table.init(users)

  // add resize handler ----------------------------------------------------------------------------
  resize()
  window.addEventListener('resize', throttle(100, resize))

  // start updating --------------------------------------------------------------------------------
  update()
  function update () {
    // advance time --------------------------------------------------------------------------------
    T.T++
    pool.contract.block = T.T

    // periodically fund pool and increment its lifetime -------------------------------------------
    pool.update()

    // increment lifetimes and ages; collect eligible claimants ------------------------------------
    const eligible: Array<User> = []
    for (const user of Object.values(users)) {
      user.update()
      if (user.claimable > 0) eligible.push(user as User)
    }

    // perform random lock/retrieve from random account for random amount --------------------------
    if (AUTO_LOCK_UNLOCK) {
      const user = pickRandom(Object.values(users))
      pickRandom([
        (amount:number)=>user.lock(amount),
        (amount:number)=>user.retrieve(amount)
      ])(random(user.balance))
    }

    // perform random claim ------------------------------------------------------------------------
    if (AUTO_CLAIM && eligible.length > 0) {
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

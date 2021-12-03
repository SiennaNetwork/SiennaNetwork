import './style/reset.css'
import './style/base.css'
import './style/layout.css'

import {Cosmos}  from './Cosmos'
import Dashboard from './Dashboard'
import {append}  from './helpers'

console.log(Cosmos.default)

Cosmos.loadContracts()
  .then(Dashboard.make)
  .then(append(document.body))

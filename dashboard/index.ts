import './style/reset.css'
import './style/base.css'
import './style/layout.css'

import {Cosmos} from './contracts/Contract'
import dashboard from './Dashboard'
import {append}  from './helpers'

console.log(Cosmos.default)

Cosmos.loadContracts()
  .then(dashboard)
  .then(append(document.body))

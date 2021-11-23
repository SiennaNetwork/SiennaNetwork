import './style/reset.css'
import './style/base.css'
import './style/layout.css'

import {initContracts} from './contracts/Contract'
import dashboard from './Dashboard'
import {append}  from './helpers'

initContracts()
  .then(dashboard)
    .then(append(document.body))

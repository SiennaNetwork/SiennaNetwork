import './style/reset.css'
import './style/base.css'
import './style/layout.css'

import contracts from './wasm'
import dashboard from './Dashboard'
import {append}  from './helpers'

contracts.then(dashboard).then(append(document.body))

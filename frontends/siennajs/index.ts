import * as contract from "./lib/contract"
import * as core from "./lib/core"
import * as token from "./lib/amm/token"
import * as exchange from "./lib/amm/exchange"
//import * as ido from "./lib/launchpad/ido"
//import * as launchpad from "./lib/launchpad/launchpad"
import * as rewards_v2 from "./lib/rewards/rewards_v2"
import * as rewards_v3 from "./lib/rewards/rewards_v3"
import * as amm_factory from "./lib/amm/amm_factory"
import * as snip20 from "./lib/snip20"
import * as hop from "./lib/amm/hop"
import * as router from "./lib/amm/router"
import * as permit from "./lib/permit"

export default {
    contract,
    core,
    token,
    exchange,
    //ido,
    //launchpad,
    rewards_v2,
    rewards_v3,
    amm_factory,
    snip20,
    hop,
    router,
    permit
}

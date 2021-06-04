/**
 * https://dev.to/craigmorten/testing-your-deno-apps-with-mocha-4f35
 * First we import Mocha and it's type definitions.
 */
// @deno-types="https://unpkg.com/@types/mocha@7.0.2/index.d.ts"
import "https://unpkg.com/mocha@8.4.0/mocha.js";
/**
 * Here I'm using the Deno `expect` library, but you can use
 * any assertion library - including Deno's std testing library.
 * See: https://deno.land/std/testing
 */
//import { expect } from "https://deno.land/x/expect@v0.2.1/mod.ts";

import SNIP20 from './SNIP20.js'
import Rewards from './Rewards.js'

import {RewardsContracts} from '../cli/ops.js'
import {abs} from '../cli/root.js'
import SecretNetwork from '../libraries/fadroma/js/SecretNetwork/index.js'

/**
 * Browser based Mocha requires `window.location` to exist.
 */
//(window as any).location = new URL("http://localhost:0");

/**
 * In order to use `describe` etc. we need to set Mocha to `bdd`
 * mode.
 * 
 * We also need to set the reporter to `spec` (though other options
 * are available) to prevent Mocha using the default browser reporter
 * which requires access to a DOM.
 */
mocha.setup({ ui: "bdd", reporter: "spec" });

/**
 * Ensure there are no leaks in our tests.
 */
mocha.checkLeaks();

/**
 * Our example function under test
 */
function add(a: number, b: number): number {
  return a + b;
}

/**
 * We write our tests as usual!
 */
describe("Rewards", () => {

  const state = {
    node:          null,
    network:       null,
    tokenCodeId:   null,
    rewardsCodeId: null,
    agent:         null,
    token:         null,
    rewards:       null
  }

  before(async function setupAll () {
    this.timeout(60000)
    // before each test run, compile fresh versions of the contracts
    const buildResult = await RewardsContracts.build({
      workspace: abs(),
      parallel: false
    })
    const {TOKEN:tokenBinary, REWARDS:rewardsBinary} = buildResult
    console.log({buildResult})
    process.exit(123)
    // run a clean localnet
    const {node, network, builder} = await SecretNetwork.localnet({
      stateBase: abs('artifacts')
    })
    Object.assign(state, { node, network, builder })
    // and upload them to it
    const receipts = await Promise.all([
      builder.uploadCached(tokenBinary),
      builder.uploadCached(rewardsBinary)
    ])
    Object.assign(state, {
      tokenCodeId:   receipts[0].codeId,
      rewardsCodeId: receipts[1].codeId
    })
    this.timeout(15000)
  })

  beforeEach(setupEach(state))

  after(cleanupAll(state))

  it("should add two positive numbers correctly", () => {
    console.log('helloOoO')
    //expect(add(2, 3)).toEqual(5);
  });
});



function setupEach (state:any) {
  return async function () {
    state.agent = await state.network.getAgent()
    state.token = await SNIP20.init({
      agent:   state.agent,
      label:   'token',
      codeId:  state.tokenCodeId,
      initMsg: RewardsContracts.contracts.TOKEN.initMsg
    })
    state.rewards = await Rewards.init({
      agent:   state.agent,
      label:   'rewards',
      codeId:  state.rewardsCodeId,
      initMsg: {
        ...RewardsContracts.contracts.REWARDS.initMsg,
        reward_token: state.token.address
      }
    })
  }
}

function cleanupAll (state:any) {
  return async function () {
    await state.node.remove()
  }
}

/**
 * And finally we run our tests, passing the onCompleted function
 * hook and setting some globals.
 */
mocha.run(onCompleted).globals(["onerror"])

/**
 * Callback on completion of tests to ensure Deno exits with
 * the appropriate status code. 
 */
function onCompleted(failures: number): void {
  if (failures > 0) {
      Deno.exit(1);
  } else {
      Deno.exit(0);
  }
}


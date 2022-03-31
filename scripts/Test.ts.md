# Sienna Scripts: Test

```typescript
import Fadroma, {
  bold, timestamp, Console, print, randomHex,
  Deployments, MigrationContext, Chain, Agent,
} from '@hackbg/fadroma'
import { b64encode } from "@waiting/base64"
import * as API from '@sienna/api'
import settings, { workspace, schedule } from '@sienna/settings'
import { deployTGE } from './Deploy'
const console = new Console('@sienna/scripts/Test')
```

## Contract unit tests

* Use `pnpm -w dev test` to run the available JavaScript integration tests.

* Use `cargo test -p $CRATE` to test individual crates,
  as listed in [/Cargo.toml](../Cargo.toml).

> **Troubleshooting:** Tests exit before they finish?
> See [/contracts/amm/router/route.test.ts.md](../contracts/router/route.test.ts.md#the-catch)
> for info and a possible workaround.

```typescript
/*import routerClientTests from '../contracts/router/test/client.test.ts.md'
commands['test'] = {}
commands['test']['router'] = {}
commands['test']['router']['client'] = routerClientTests
commands['test']['router']['integration'] = async () => {
  const tests = await import('../contracts/router/test/integration.test.ts.md')
  await tests.allDone
}*/
```

## Contract client tests

This makes sure each client can be constructed,
and thus checks there are no "shallow" errors, e.g.
syntax errors, broken module imports/exports.

```typescript
Fadroma.command('clients', () => {
  new API.SiennaSnip20Client()
  new API.MGMTClient()
  new API.RPTClient()
  new API.AMMFactoryClient['v1']()
  new API.AMMFactoryClient['v2']()
  new API.AMMExchangeClient['v1']()
  new API.AMMExchangeClient['v2']()
  new API.AMMSnip20Client()
  new API.LPTokenClient()
  new API.RewardsClient['v2']()
  new API.RewardsClient['v3']()

  // TODO: these don't have clients yet
  new API.LaunchpadClient()
  new API.IDOClient()
  new API.InterestModelClient()
  new API.LendMarketClient()
  new API.LendOracleClient()
  new API.LendOverseerClient()
}
```

## Fund testers

Send some testnet SIENNA from the active deployment
to pre-defined addresses.

```typescript
Fadroma.command('fund',
  Deployments.activate,
  () => { throw new Error('not implemented') }
)
```

## Integration test

This is a multi-stage integration test covering the migration
from Sienna AMM v1 + Sienna Rewards v2
to Sienna AMM v2 and Sienna Rewards v3.
This involves recreating all the AMM and rewards contracts.

### Integration test steps

```typescript
const integrationTest = {

  setup: async function integrationTestSetup ({ chain: { isDevnet }, agent: { address } }) {
    if (!isDevnet) {
      throw new Error('@sienna/mgmt: This command is for devnet only.')
    }
    const scheduleMod = JSON.parse(JSON.stringify(schedule))
    console.warn('Redirecting MintingPool/LPF to admin balance. Only run this on devnet.')
    scheduleMod.pools[5].accounts[0].address = address
    console.warn('Changing RPT to vest every 10 seconds. Only run this on devnet.')
    scheduleMod.pools[5].accounts[1].interval = 10
    console.warn('Setting viewing key of agent to empty string.')
    return { schedule: scheduleMod }
  },

  claim: async function integrationTestClaim ({
    agent, deployment,
    MGMT    = new MGMTClient({ ...deployment.get('MGMT'), agent })
  }) {
    console.warn('Integration test: claiming from LPF')
    await MGMT.tx().claim()
  },

  getLPTokens: v => async function integrationTestGetLPTokens ({
    agent, deployment,
    FACTORY = new AMMFactoryClient [v] ({ ...deployment.get(`AMM[${v}].Factory`), agent })
    SIENNA  = new SiennaSnip20Client   ({ ...deployment.get('SIENNA'),            agent })
    SSCRT   = new Snip20Client         ({ ...deployment.get('Placeholder.sSCRT'), agent })
  }) {
    const { EXCHANGE, LP_TOKEN } = await FACTORY.getExchange(SIENNA.asCustomToken, SSCRT.asCustomToken)
    await agent.bundle(async agent=>{
      await SIENNA.tx(agent).setViewingKey("")
      await LP_TOKEN.tx(agent).setViewingKey("")
    })
    console.info(bold('Initial LP token balance:'), await LP_TOKEN.q().balance(agent.address, ""))
    await agent.bundle(async agent=>{
      await SIENNA.tx(agent).increaseAllowance("1000", EXCHANGE.address)
      await SSCRT.tx(agent).increaseAllowance("1000", EXCHANGE.address)
      await EXCHANGE.tx(agent).add_liquidity({
        token_0: SIENNA.asCustomToken,
        token_1: SSCRT.asCustomToken
      }, "1000", "1000")
    })
    console.info(bold('New LP token balance:'), await LP_TOKEN.q().balance(agent.address, ""))
    return { EXCHANGE, LP_TOKEN, SIENNA }
  },

  stakeLPTokens: v => async function integrationTestStakeLPTokens ({
    agent, deployment,
    SIENNA  = new SiennaSnip20Client({ ...deployment.get('SIENNA'),                     agent }),
    RPT     = new RPTClient         ({ ...deployment.get('RPT'),                        agent }),
    SSSSS   = new RewardsClient [v] ({ ...deployment.get(`Rewards[${v}].SSSSS`),        agent }),
    REWARDS = new RewardsClient [v] ({ ...deployment.get(`Rewards[${v}].SIENNA-SSCRT`), agent })
  }) {
    console.info(bold('Initial SIENNA balance:'), await SIENNA.q().balance(agent.address, ""))
    const LP_TOKEN = await REWARDS.lpToken()
    await agent.bundle(async agent=>{
      await LP_TOKEN.tx(agent).increaseAllowance("100", REWARDS.address)
      await REWARDS.tx(agent).lock("100")
      await SIENNA.tx(agent).increaseAllowance("100", SSSSS.address)
      await SSSSS.tx(agent).lock("100")
    })
    console.info(bold('SIENNA balance after staking:'), await SIENNA.q().balance(agent.address, ""))
    await agent.bundle(async agent=>{
      await RPT.tx(agent).vest()
      await SSSSS.tx(agent).set_viewing_key("")
      await REWARDS.tx(agent).set_viewing_key("")
    })
    console.info(await Promise.all([SSSSS.q(agent).pool_info(), SSSSS.q(agent).user_info()]))
    try {
      await SSSSS.tx(agent).claim()
    } catch (e) {
      console.error(bold(`Could not claim from SSSSS ${v}:`, e.message))
    }
    console.info(await Promise.all([REWARDS.q(agent).pool_info(), REWARDS.q(agent).user_info()]))
    try {
      await REWARDS.tx(agent).claim()
    } catch (e) {
      console.error(bold(`Could not claim from Rewards ${v}:`, e.message))
    }
    console.info(bold('SIENNA balance after claiming:'), await SIENNA.q().balance(agent.address, ""))
  },

  vestV3: async function integrationTestVestV3 ({
    agent, deployment,
    RPT     = new RPTClient({...deployment.get('RPT'), agent})
    SSSSS   = new RewardsClient.v3({...deployment.get(`Rewards[v3].SSSSS`),        agent})
    REWARDS = new RewardsClient.v3({...deployment.get(`Rewards[v3].SIENNA-SSCRT`), agent})
  }) {
    console.info('Before vest', await Promise.all([SSSSS.q(agent).user_info(), REWARDS.q(agent).user_info()]))
    await RPT.tx(agent).vest()
    console.info('After vest', await Promise.all([SSSSS.q(agent).user_info(), REWARDS.q(agent).user_info()]))
    await agent.bundle(async agent=>{ await SSSSS.tx(agent).epoch() await REWARDS.tx(agent).epoch()})
    console.info('After epoch', await Promise.all([SSSSS.q(agent).user_info(), REWARDS.q(agent).user_info()]))
    await agent.bundle(async agent=>{await SSSSS.tx(agent).claim() await REWARDS.tx(agent).claim()})
    console.info('After claim', await Promise.all([SSSSS.q(agent).user_info(), REWARDS.q(agent).user_info()]))
  }

}
```

## Sienna Lend tests

```typescript
Fadroma.command('lend', Deployments.activate, testLend)
export async function testLend({
  chain,
  agent,
  deployment,
  prefix,
}: MigrationContext) {

  async function withGasReport(agent: Agent, contract: any, msg: any) {
    let op = Object.keys(msg)[0];
    let res = await agent.execute(contract, msg);
    gasTable.push({ op, gas_wanted: res.gas_wanted, gas_used: res.gas_used });
  }

  const [ALICE, BOB, MALLORY] = await Promise.all([
    chain.getAgent("ALICE"),
    chain.getAgent("BOB"),
    chain.getAgent("MALLORY")
  ])

  const TOKEN1 = new AMMSNIP20Contract({ workspace, name: "SLATOM" });
  const TOKEN2 = new AMMSNIP20Contract({ workspace, name: "SLSCRT" });
  await chain.buildAndUpload(agent, [TOKEN1, TOKEN2]);
  const token1 = await deployment.get(TOKEN1.name, "SLATOM");
  const token2 = await deployment.get(TOKEN2.name, "SLSCRT");

  const gasTable = [];

  const INTEREST_MODEL        = new InterestModelContract({ workspace });
  const deployedInterestModel = await deployment.get(INTEREST_MODEL.name);

  const OVERSEER         = new LendOverseerContract({ workspace });
  const deployedOverseer = await deployment.get(OVERSEER.name);

  const MOCK_ORACLE        = new MockOracleContract({ workspace });
  const deployedMockOracle = await deployment.get(MOCK_ORACLE.name);

  // set prices
  await agent.execute(deployedMockOracle, { set_price: { symbol: "SLATOM", price: "1" } });
  await agent.execute(deployedMockOracle, { set_price: { symbol: "SLSCRT", price: "1" } });

  console.info("minting tokens...");

  await withGasReport(agent, token1, {
    mint: { recipient: BOB.address, amount: "100" },
  });

  await withGasReport(agent, token1, {
    mint: { recipient: MALLORY.address, amount: "100" },
  });

  await withGasReport(agent, token2, {
    mint: { recipient: ALICE.address, amount: "300" },
  });

  console.info("listing markets...");
  await withGasReport(agent, deployedOverseer, {
    whitelist: {
      config: {
        config: {
          initial_exchange_rate: "0.2",
          reserve_factor: "1",
          seize_factor: "0.9",
        },
        entropy: randomHex(36),
        interest_model_contract: {
          address: deployedInterestModel.address,
          code_hash: deployedInterestModel.codeHash,
        },
        ltv_ratio: "0.7",
        prng_seed: randomHex(36),
        token_symbol: "SLATOM",
        underlying_asset: {
          address: token1.address,
          code_hash: token1.codeHash,
        },
      },
    },
  });

  await withGasReport(agent, deployedOverseer, {
    whitelist: {
      config: {
        config: {
          initial_exchange_rate: "0.2",
          reserve_factor: "1",
          seize_factor: "0.9",
        },
        entropy: randomHex(36),
        interest_model_contract: {
          address: deployedInterestModel.address,
          code_hash: deployedInterestModel.codeHash,
        },
        ltv_ratio: "0.7",
        prng_seed: randomHex(36),
        token_symbol: "SLSCRT",
        underlying_asset: {
          address: token2.address,
          code_hash: token2.codeHash,
        },
      },
    },
  });

  let [market1, market2] = await agent.query(deployedOverseer, {
    markets: { pagination: { start: 0, limit: 10 } },
  });

  console.info("depositing...");
  await withGasReport(BOB, token1, {
    send: {
      recipient: market1.contract.address,
      recipient_code_hash: market1.contract.code_hash,
      amount: "100",
      msg: b64encode(JSON.stringify("deposit")),
    },
  });

  await withGasReport(ALICE, token2, {
    send: {
      recipient: market2.contract.address,
      recipient_code_hash: market2.contract.code_hash,
      amount: "300",
      msg: b64encode(JSON.stringify("deposit")),
    },
  });

  console.info("entering markets...");
  await withGasReport(BOB, deployedOverseer, {
    enter: {
      markets: [market1.contract.address, market2.contract.address],
    },
  });

  await withGasReport(ALICE, deployedOverseer, {
    enter: {
      markets: [market1.contract.address, market2.contract.address],
    },
  });

  await withGasReport(MALLORY, deployedOverseer, {
    enter: {
      markets: [market1.contract.address, market2.contract.address],
    },
  });

  console.info("borrowing...");
  await withGasReport(BOB, market2.contract, {
    borrow: {
      amount: "100",
    },
  });

  await withGasReport(MALLORY, market2.contract, {
    borrow: {
      amount: "100",
    },
  });

  console.info("repaying...");
  await withGasReport(BOB, token2, {
    send: {
      recipient: market2.contract.address,
      recipient_code_hash: market2.contract.code_hash,
      amount: "100",
      msg: b64encode(JSON.stringify({ repay: { borrower: null } })),
    },
  });

  console.table(gasTable, ["op", "gas_wanted", "gas_used"]);
}
```

## Entry point

```typescript
Error.stackTraceLimit = Infinity
export default Fadroma.module(import.meta.url)
```
